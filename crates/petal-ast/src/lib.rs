mod definition;
mod expression;
mod statement;

pub use definition::*;
pub use expression::*;
use petal_diagnostic::{Diagnostic, DiagnosticSeverity};
use petal_lexer::{Cursor, Token, TokenCursor, TokenKind};
use petal_span::Span;
pub use statement::*;

struct Parser<'a, 'd> {
    source: &'a str,

    /// The iterator to consume tokens from.
    cursor: TokenCursor<'a>,

    /// The [`Vec`] of [`Diagnostic`]s to push to.
    diagnostics: &'d mut Vec<Diagnostic>,
}

impl<'a, 'd> Parser<'a, 'd> {
    /// Create a new [`Parser`] consuming the provided [`Iterator`] of [`Token`]s.
    pub fn new(
        source: &'a str,
        tokens: impl Iterator<Item = Token> + 'a,
        diagnostics: &'d mut Vec<Diagnostic>,
    ) -> Self {
        Self {
            source,
            cursor: TokenCursor::new(tokens, /* skip_comments */ true),
            diagnostics,
        }
    }
}

impl Parser<'_, '_> {
    /// Expects a token of a certain kind to be at the parser's current position. If the token matches the expected
    /// kind, it will be consumed.
    ///
    /// If the token at the parser's current position does not match the expected kind, an error diagnostic will be
    /// produced, and [`None`] will be returned.
    fn expect(&mut self, kind: TokenKind) -> Option<Token> {
        let Some(token) = self.cursor.peek() else {
            // TODO: use the last span that we visited
            self.diagnostics.push(Diagnostic::error(
                Span::default(),
                format!("expected token '{kind:?}' but reached the end of the file"),
            ));

            return None;
        };

        if token.kind != kind {
            self.diagnostics.push(Diagnostic::error(
                token.span,
                format!("expected token '{kind:?}' but got '{:?}'", token.kind),
            ));

            return None;
        }

        self.cursor.consume()
    }
}

impl Parser<'_, '_> {
    /// Return the definition at the parser's current position, advancing its cursor.
    pub fn next_definition(&mut self) -> Option<Definition> {
        // If we have reached the end of the stream, then there is nothing else to parse.
        self.cursor.peek()?;

        // TODO: If `None` is returned from this, we should consume tokens until we reach a token that is valid for a
        //       definition (i.e. in this case, another `func` keyword).
        self.next_function_definition().map(Definition::Function)
    }

    /// Parse a function definition at the parser's current position.
    fn next_function_definition(&mut self) -> Option<FunctionDefinition> {
        // The first token must be an identifier of `func`.
        let identifier_token = self.expect(TokenKind::Identifier)?;
        if identifier_token.span.slice(self.source) != "func" {
            return None;
        }

        // Then, there must be the name of the function.
        let name_token = self.expect(TokenKind::Identifier)?;

        // Then, there must be an opening parenthesis, followed by a closing parenthesis
        self.expect(TokenKind::OpenParen)?;
        self.expect(TokenKind::CloseParen)?;

        // There may be an arrow, which indicates that a return type is being specified.
        let return_type_name = if self.cursor.consume_if(|it| it.kind == TokenKind::Arrow).is_some() {
            let return_type_name_token = self.expect(TokenKind::Identifier)?;
            Some(return_type_name_token.span.slice(self.source).to_string())
        } else {
            None
        };

        // Then, there must be a block.
        let body = self.next_block()?;
        let span = identifier_token.span.until(body.span);

        Some(FunctionDefinition {
            name: name_token.span.slice(self.source).to_string(),
            return_type_name,
            body,
            span,
        })
    }

    /// Parse a block at the parser's current position.
    fn next_block(&mut self) -> Option<Block> {
        // The block must start with an opening brace.
        let open_brace = self.expect(TokenKind::OpenBrace)?;

        let mut statements: Vec<Statement> = Vec::new();

        while self.cursor.peek().map_or_default(|it| it.kind != TokenKind::CloseBrace) {
            statements.push(self.next_statement()?);
        }

        // And end with a closing brace.
        let close_brace = self.expect(TokenKind::CloseBrace)?;

        Some(Block {
            statements,
            span: open_brace.span.until(close_brace.span),
        })
    }
}

impl Parser<'_, '_> {
    /// Parse a statement at the parser's current position.
    fn next_statement(&mut self) -> Option<Statement> {
        let statement = self.next_return_statement().map(Statement::Return)?;

        self.expect(TokenKind::Semicolon)?;

        Some(statement)
    }

    /// Parse a return statement at the parser's current position.
    fn next_return_statement(&mut self) -> Option<ReturnStatement> {
        let keyword_token = self.expect(TokenKind::Identifier)?;
        let keyword_str = keyword_token.span.slice(self.source);

        if keyword_str != "return" {
            self.diagnostics.push(Diagnostic::error(
                keyword_token.span,
                format!("expected 'return' keyword, but got '{keyword_str}'"),
            ));

            return None;
        }

        // If the next token is a semicolon, then there is no value being returned.
        if self.cursor.peek().map_or_default(|it| it.kind == TokenKind::Semicolon) {
            return Some(ReturnStatement {
                value: None,
                span: keyword_token.span,
            });
        }

        let value = self.parse_expression()?;
        let span = keyword_token.span.until(value.span());

        Some(ReturnStatement {
            value: Some(value),
            span,
        })
    }
}

impl Parser<'_, '_> {
    /// Parse an expression at the parser's current position.
    fn parse_expression(&mut self) -> Option<Expression> {
        self.parse_number_expression()
    }

    /// Parse a number expression at the parser's current position.
    fn parse_number_expression(&mut self) -> Option<Expression> {
        let token = self.expect(TokenKind::Number)?;
        let string = token.span.slice(self.source);

        let is_float = string.contains('.');
        if is_float {
            self.diagnostics.push(Diagnostic::error(
                token.span,
                "floating point number literals are not supported yet",
            ));

            return None;
        }

        let Ok(int) = string.parse::<i64>() else {
            self.diagnostics.push(Diagnostic::error(
                token.span,
                format!("number literal '{string}' could not be parsed as an i64"),
            ));

            return None;
        };

        Some(Expression::IntegerLiteral {
            value: int,
            span: token.span,
        })
    }
}

/// The result of calling [`parse`].
#[derive(Default)]
pub struct ParseResult {
    /// The definitions parsed from the source.
    pub definitions: Vec<Definition>,

    /// The diagnostics produced while parsing the source.
    pub diagnostics: Vec<Diagnostic>,
}

impl ParseResult {
    /// Return whether the parsing can continue (i.e. whether there are any error diagnostics in the result).
    pub fn can_continue(&self) -> bool {
        self.diagnostics
            .iter()
            .all(|it| it.severity != DiagnosticSeverity::Error)
    }
}

/// Parse definitions from the provided [`Iterator`] of [`Token`]s.
pub fn parse<'a>(source: &'a str, tokens: impl Iterator<Item = Token> + 'a) -> ParseResult {
    let mut result = ParseResult::default();

    let mut parser = Parser::new(source, tokens, &mut result.diagnostics);
    while let Some(definition) = parser.next_definition() {
        result.definitions.push(definition);
    }

    result
}

#[cfg(test)]
mod test {
    use super::*;

    fn tokenize_and_parse(source: &str) -> ParseResult {
        let tokens = petal_lexer::tokenize(source);
        parse(source, tokens)
    }

    #[test]
    fn parses_function_definition() {
        let result = tokenize_and_parse("func main() {}");

        assert!(result.can_continue());

        assert_eq!(result.diagnostics, vec![]);

        assert_eq!(result.definitions, vec![Definition::Function(FunctionDefinition {
            name: "main".to_string(),
            return_type_name: None,
            body: Block {
                statements: vec![],
                span: Span::new(12, 14)
            },
            span: Span::new(0, 14)
        })]);
    }

    #[test]
    fn parses_function_definition_with_return_type() {
        let result = tokenize_and_parse("func main() -> i32 {}");

        assert!(result.can_continue());

        assert_eq!(result.diagnostics, vec![]);

        assert_eq!(result.definitions, vec![Definition::Function(FunctionDefinition {
            name: "main".to_string(),
            return_type_name: Some("i32".to_string()),
            body: Block {
                statements: vec![],
                span: Span::new(19, 21)
            },
            span: Span::new(0, 21)
        })]);
    }

    #[test]
    fn parses_empty_return_statement() {
        let result = tokenize_and_parse("func main() { return; }");

        assert!(result.can_continue());

        assert_eq!(result.diagnostics, vec![]);

        assert_eq!(result.definitions, vec![Definition::Function(FunctionDefinition {
            name: "main".to_string(),
            return_type_name: None,
            body: Block {
                statements: vec![Statement::Return(ReturnStatement {
                    value: None,
                    span: Span::new(14, 20)
                })],
                span: Span::new(12, 23)
            },
            span: Span::new(0, 23)
        })]);
    }

    #[test]
    fn parses_return_statement_with_integer_literal() {
        let result = tokenize_and_parse("func main() { return 123; }");

        assert!(result.can_continue());

        assert_eq!(result.diagnostics, vec![]);

        assert_eq!(result.definitions, vec![Definition::Function(FunctionDefinition {
            name: "main".to_string(),
            return_type_name: None,
            body: Block {
                statements: vec![Statement::Return(ReturnStatement {
                    value: Some(Expression::IntegerLiteral {
                        value: 123,
                        span: Span::new(21, 24)
                    }),
                    span: Span::new(14, 24)
                })],
                span: Span::new(12, 27)
            },
            span: Span::new(0, 27)
        })]);
    }

    #[test]
    fn cannot_parse_function_definition_without_body() {
        let result = tokenize_and_parse("func main()");

        assert!(!result.can_continue());

        assert_eq!(result.definitions, vec![]);

        assert_eq!(result.diagnostics, vec![Diagnostic::error(
            // TODO: When this works properly, it should be the span of `)`.
            Span::default(),
            "expected token 'OpenBrace' but reached the end of the file"
        )]);
    }

    #[test]
    fn cannot_parse_function_definition_with_incomplete_return_type() {
        let result = tokenize_and_parse("func main() -> {}");

        assert!(!result.can_continue());

        assert_eq!(result.definitions, vec![]);

        assert_eq!(result.diagnostics, vec![Diagnostic::error(
            Span::new(15, 16),
            "expected token 'Identifier' but got 'OpenBrace'"
        )]);
    }

    #[test]
    fn cannot_parse_empty_return_statement_without_semicolon() {
        let result = tokenize_and_parse("func main() { return }");

        assert!(!result.can_continue());

        assert_eq!(result.definitions, vec![]);

        assert_eq!(result.diagnostics.len(), 1);

        // We don't expect a specific message, but we do expect the diagnostic to be at the curly brace's position,
        // indicating that it attempted to parse a value.
        let diagnostic = &result.diagnostics[0];
        assert_eq!(diagnostic.severity, DiagnosticSeverity::Error);
        assert_eq!(diagnostic.span, Span::new(21, 22));
    }

    #[test]
    fn cannot_parse_return_statement_with_integer_literal_without_semicolon() {
        let result = tokenize_and_parse("func main() { return 123 }");

        assert!(!result.can_continue());

        assert_eq!(result.definitions, vec![]);

        assert_eq!(result.diagnostics, vec![Diagnostic::error(
            Span::new(25, 26),
            "expected token 'Semicolon' but got 'CloseBrace'"
        )]);
    }
}
