use crate::{
    ast::{
        error::{
            ASTError,
            ASTErrorKind,
        },
        expression::{
            Expression,
            ExpressionKind,
        },
        statement::{
            Statement,
            function_declaration::FunctionDeclaration,
            r#return::Return,
        },
    },
    core::span::Span,
    lexer::token::{
        Keyword,
        Token,
        TokenKind,
    },
};

pub mod error;
pub mod expression;
pub mod statement;

/// The AST parser.
pub struct ASTParser {
    /// The tokens to parse into an AST.
    tokens: Vec<Token>,

    /// The position that the parser is at within the [tokens].
    cursor: usize,
}

impl ASTParser {
    /// Creates a new [ASTParser].
    pub fn new(tokens: Vec<Token>) -> Self {
        ASTParser { tokens, cursor: 0 }
    }

    /// Creates a new [ASTParser] instance and parses all of the provided [tokens] into an AST.
    pub fn new_and_parse(tokens: Vec<Token>) -> Result<Vec<Statement>, ASTError> {
        let mut parser = ASTParser::new(tokens);
        parser.parse()
    }

    /// Attempts to parse the [tokens] within this [ASTParser] into an AST.
    pub fn parse(&mut self) -> Result<Vec<Statement>, ASTError> {
        let mut statements: Vec<Statement> = vec![];

        while let Some(token) = self.peek() {
            let statement: Statement = match token.kind {
                TokenKind::Keyword(Keyword::Func) => self.parse_function_declaration()?,
                _ => return Err(ASTErrorKind::UnexpectedToken(token.kind.clone()).at(token.span)),
            };

            statements.push(statement);
        }

        Ok(statements)
    }

    /// Attempts to parse a statement at the [ASTParser]'s current position.
    fn parse_statement(&mut self) -> Result<Statement, ASTError> {
        let token = self.peek_expect_any()?;

        let statement = match token.kind {
            TokenKind::Keyword(Keyword::Return) => self.parse_return()?,
            _ => return Err(ASTErrorKind::ExpectedStatement(token.kind.clone()).at(token.span)),
        };

        self.expect(TokenKind::Semicolon)?;

        Ok(statement)
    }

    /// Attempts to parse an expression at the [ASTParser]'s current position.
    fn parse_expression(&mut self) -> Result<Expression, ASTError> {
        let token = self.peek_expect_any()?;

        let expression = match token.kind {
            TokenKind::Number(value) => {
                // FIXME: We need to copy the span before attempting to acquire a mutable reference via consume.
                let span = token.span;
                self.consume();
                Expression::new(ExpressionKind::NumberLiteral(value), span)
            }

            _ => return Err(ASTErrorKind::ExpectedExpression(token.kind.clone()).at(token.span)),
        };

        Ok(expression)
    }

    /// Attempts to parse a function declaration from the [ASTParser]'s current position.
    fn parse_function_declaration(&mut self) -> Result<Statement, ASTError> {
        // All functions must start with the func keyword.
        let func_keyword_span = self.expect_span(TokenKind::Keyword(Keyword::Func))?;

        // Then, the name of the function must be present.
        let (function_name, _) = self.expect_identifier()?;

        // Then parenthesis must surround the parameters to the function.
        self.expect(TokenKind::OpenParen)?;
        self.expect(TokenKind::CloseParen)?;

        // And braces must surround the body of the function.
        self.expect(TokenKind::OpenBrace)?;

        let mut body: Vec<Statement> = Vec::new();

        while !self.peek().map(|it| it.kind == TokenKind::CloseBrace).unwrap_or(true) {
            body.push(self.parse_statement()?);
        }

        let closing_brace_span = self.expect_span(TokenKind::CloseBrace)?;

        Ok(Statement::new(
            FunctionDeclaration::new(function_name, body).into(),
            Span::between(func_keyword_span, closing_brace_span),
        ))
    }

    /// Attempts to parse a return statement from the [ASTParser]'s current position.
    fn parse_return(&mut self) -> Result<Statement, ASTError> {
        let return_keyword_span = self.expect_span(TokenKind::Keyword(Keyword::Return))?;

        if self.peek().map(|it| it.kind == TokenKind::Semicolon).unwrap_or_default() {
            return Ok(Statement::from(Return::new(None), return_keyword_span));
        }

        let value = self.parse_expression()?;
        let span = Span::between(return_keyword_span, value.span);
        Ok(Statement::from(Return::new(Some(value)), span))
    }

    /// Returns the token at the [ASTParser]'s current position.
    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.cursor)
    }

    /// Returns the token at the [ASTParser]'s current position, advancing the cursor.
    fn consume(&mut self) -> Option<&Token> {
        self.tokens.get(self.cursor).inspect(|_| self.cursor += 1)
    }

    /// Expects a token to be at the [ASTParser]'s current position, advancing the cursor.
    /// An [ASTErrorKind::UnexpectedEndOfFile] will be returned if there are no tokens left in the stream.
    fn expect_any(&mut self) -> Result<&Token, ASTError> {
        let last_token_span = self.tokens.last().map(|it| it.span).unwrap_or_default();
        self.consume().ok_or(ASTErrorKind::UnexpectedEndOfFile.at(last_token_span))
    }

    /// Expects a token to be at the [ASTParser]'s current position, advancing the cursor.
    /// An [ASTErrorKind::UnexpectedEndOfFile] will be returned if there are no tokens left in the stream.
    fn peek_expect_any(&self) -> Result<&Token, ASTError> {
        let last_token_span = self.tokens.last().map(|it| it.span).unwrap_or_default();
        self.peek().ok_or(ASTErrorKind::UnexpectedEndOfFile.at(last_token_span))
    }

    /// Expects a certain token to be at the [ASTParser]'s current position, advancing the cursor.
    fn expect(&mut self, kind: TokenKind) -> Result<&Token, ASTError> {
        let token = self.expect_any()?;
        if token.kind != kind {
            return Err(ASTErrorKind::ExpectedToken { expected: kind, got: token.kind.clone() }.at(token.span));
        }

        Ok(token)
    }

    /// Like [expect], but only returns the matched token's span.
    fn expect_span(&mut self, kind: TokenKind) -> Result<Span, ASTError> {
        self.expect(kind).map(|it| it.span)
    }

    /// Expects an identifier token to be at the [ASTParser]'s current position, advancing the cursor.
    fn expect_identifier(&mut self) -> Result<(String, Span), ASTError> {
        let token = self.expect_any()?;
        match &token.kind {
            TokenKind::Identifier(identifier) => Ok((identifier.into(), token.span)),
            _ => Err(ASTErrorKind::ExpectedIdentifier.at(token.span)),
        }
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::{
        core::span::Span,
        lexer::token::{
            Keyword,
            TokenKind,
        },
    };

    fn assert_ast_error(tokens: Vec<Token>, error: ASTError) {
        assert_eq!(ASTParser::new_and_parse(tokens), Err(error))
    }

    #[test]
    fn parse_function_declaration() {
        insta::assert_debug_snapshot!(ASTParser::new_and_parse(vec![
            Token::new(TokenKind::Keyword(Keyword::Func), Span { start: 0, length: 4 }),
            Token::new(TokenKind::Identifier("main".into()), Span { start: 5, length: 4 }),
            Token::new(TokenKind::OpenParen, Span { start: 10, length: 1 }),
            Token::new(TokenKind::CloseParen, Span { start: 11, length: 1 }),
            Token::new(TokenKind::OpenBrace, Span { start: 12, length: 1 }),
            Token::new(TokenKind::CloseBrace, Span { start: 13, length: 1 }),
        ]), @r#"
        Ok(
            [
                Statement {
                    kind: FunctionDeclaration(
                        FunctionDeclaration {
                            name: "main",
                            body: [],
                        },
                    ),
                    span: Span {
                        start: 0,
                        length: 14,
                    },
                },
            ],
        )
        "#);
    }

    #[test]
    fn parse_function_declaration_with_return_statement() {
        insta::assert_debug_snapshot!(ASTParser::new_and_parse(vec![
            Token::new(TokenKind::Keyword(Keyword::Func), Span { start: 0, length: 4 }),
            Token::new(TokenKind::Identifier("main".into()), Span { start: 5, length: 4 }),
            Token::new(TokenKind::OpenParen, Span { start: 10, length: 1 }),
            Token::new(TokenKind::CloseParen, Span { start: 11, length: 1 }),
            Token::new(TokenKind::OpenBrace, Span { start: 12, length: 1 }),
            Token::new(TokenKind::Keyword(Keyword::Return), Span { start: 13, length: 4 }),
            Token::new(TokenKind::Number(1234.0), Span { start: 17, length: 4 }),
            Token::new(TokenKind::Semicolon, Span { start: 21, length: 1 }),
            Token::new(TokenKind::CloseBrace, Span { start: 22, length: 1 }),
        ]), @r#"
        Ok(
            [
                Statement {
                    kind: FunctionDeclaration(
                        FunctionDeclaration {
                            name: "main",
                            body: [
                                Statement {
                                    kind: Return(
                                        Return {
                                            value: Some(
                                                Expression {
                                                    kind: NumberLiteral(
                                                        1234.0,
                                                    ),
                                                    span: Span {
                                                        start: 17,
                                                        length: 4,
                                                    },
                                                },
                                            ),
                                        },
                                    ),
                                    span: Span {
                                        start: 13,
                                        length: 8,
                                    },
                                },
                            ],
                        },
                    ),
                    span: Span {
                        start: 0,
                        length: 23,
                    },
                },
            ],
        )
        "#);
    }

    #[test]
    fn error_at_unexpected_end_of_file() {
        assert_ast_error(
            vec![
                Token::new(TokenKind::Keyword(Keyword::Func), Span { start: 0, length: 4 }),
                Token::new(TokenKind::Identifier("main".into()), Span { start: 5, length: 4 }),
            ],
            ASTErrorKind::UnexpectedEndOfFile.at(Span { start: 5, length: 4 }),
        );
    }
}
