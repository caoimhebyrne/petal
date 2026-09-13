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
            cursor: TokenCursor::new(tokens),
            diagnostics,
        }
    }
}

impl Parser<'_, '_> {
    /// Expects the token at the parser's current position to pass the provided `predicate`.
    ///
    /// If the `predicate` returns false for the token, the `message` will be added as an error diagnostic, and [`None`]
    /// will be returned.
    fn expect<S>(
        &mut self,
        predicate: impl FnOnce(&Token) -> bool,
        message: impl FnOnce(Option<&Token>) -> S,
    ) -> Option<Token>
    where
        S: Into<String>,
    {
        // TODO: I feel like this belongs somewhere else. I'm just not sure where that is at the moment.
        self.cursor.consume_while(|it| it.kind == TokenKind::Comment);

        let Some(token) = self.cursor.peek() else {
            self.diagnostics.push(Diagnostic::error(Span::default(), message(None)));
            return None;
        };

        if !predicate(&token) {
            self.diagnostics
                .push(Diagnostic::error(Span::default(), message(Some(&token))));

            return None;
        }

        self.cursor.consume();
        Some(token)
    }

    /// Expects a token of a certain kind to be at the parser's current position. If the token matches the expected
    /// kind, it will be consumed.
    ///
    /// If the token at the parser's current position does not match the expected kind, an error diagnostic will be
    /// produced, and [`None`] will be returned.
    fn expect_kind(&mut self, kind: TokenKind) -> Option<Token> {
        self.expect(
            |it| it.kind == kind,
            |maybe| {
                let got = match maybe {
                    Some(it) => format!("'{:?}'", it.kind),
                    None => "end of file".to_string(),
                };

                // // TODO: Implement `Display` on `TokenKind`.
                format!("expected token '{kind:?}', but got {got:?}")
            },
        )
    }

    /// Expects any number token to be at the parser's current position. If the token matches the expected kind, it will
    /// be consumed, and whether it is a floating point or not will be returned.
    ///
    /// If the token at the parser's current position does not match the expected kind, an error diagnostic will be
    /// produced, and [`None`] will be returned.
    fn expect_number(&mut self) -> Option<(bool, Span)> {
        let token = self.expect(
            |it| matches!(it.kind, TokenKind::Number { .. }),
            |maybe| {
                let got = match maybe {
                    // // TODO: Implement `Display` on `TokenKind`.
                    Some(it) => format!("'{:?}'", it.kind),
                    None => "end of file".to_string(),
                };

                format!("expected a number, but got {got:?}")
            },
        )?;

        let TokenKind::Number { float } = token.kind else {
            panic!("predicate of `expect` yielded a token that was not of the same type");
        };

        Some((float, token.span))
    }
}

impl Parser<'_, '_> {
    /// Return the definition at the parser's current position, advancing its cursor.
    pub fn next_definition(&mut self) -> Option<Definition> {
        // TODO: I don't like this.
        self.cursor.peek()?;

        // TODO: If `None` is returned from this, we should consume tokens until we reach a token that is valid for a
        //       definition (i.e. in this case, another `func` keyword).
        self.next_function_definition().map(Definition::Function)
    }

    /// Parse a function definition at the parser's current position.
    fn next_function_definition(&mut self) -> Option<FunctionDefinition> {
        // The first token must be an identifier of `func`.
        let identifier_token = self.expect_kind(TokenKind::Identifier)?;
        if identifier_token.span.slice(self.source) != "func" {
            return None;
        }

        // Then, there must be the name of the function.
        let name_token = self.expect_kind(TokenKind::Identifier)?;

        // Then, there must be an opening parenthesis, followed by a closing parenthesis
        self.expect_kind(TokenKind::OpenParen)?;
        self.expect_kind(TokenKind::CloseParen)?;

        // There may be an arrow, which indicates that a return type is being specified.
        let return_type_name = if self.cursor.consume_if(|it| it.kind == TokenKind::Arrow).is_some() {
            let return_type_name_token = self.expect_kind(TokenKind::Identifier)?;
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
        let open_brace = self.expect_kind(TokenKind::OpenBrace)?;

        let mut statements: Vec<Statement> = Vec::new();

        while self.cursor.peek().map_or_default(|it| it.kind != TokenKind::CloseBrace) {
            statements.push(self.next_statement()?);
        }

        // And end with a closing brace.
        let close_brace = self.expect_kind(TokenKind::CloseBrace)?;

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

        self.expect_kind(TokenKind::Semicolon)?;

        Some(statement)
    }

    /// Parse a return statement at the parser's current position.
    fn next_return_statement(&mut self) -> Option<ReturnStatement> {
        let keyword_token = self.expect_kind(TokenKind::Identifier)?;
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
        let (float, span) = self.expect_number()?;
        let string = span.slice(self.source);

        if float {
            self.diagnostics.push(Diagnostic::error(
                span,
                "floating point number literals are not supported yet",
            ));

            return None;
        }

        let Ok(int) = string.parse::<i64>() else {
            self.diagnostics.push(Diagnostic::error(
                span,
                format!("number literal '{string}' could not be parsed as an i64"),
            ));

            return None;
        };

        Some(Expression::IntegerLiteral { value: int, span })
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
