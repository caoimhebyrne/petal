use crate::{
    ast::{
        error::ASTErrorKind,
        expression::{Expression, ExpressionKind},
        statement::{Statement, VariableDeclaration},
    },
    core::{error::Error, source_span::SourceSpan},
    lexer::{
        Lexer,
        token::{Keyword, Token, TokenKind},
    },
};

pub mod error;
pub mod expression;
pub mod statement;

/// Converts tokens from a [Lexer] into an Abstract Syntax Tree.
pub struct ASTParser<'a> {
    /// The lexer to read tokens from.
    lexer: &'a mut Lexer<'a>,
}

impl<'a> ASTParser<'a> {
    /// Creates a new [ASTParser] which reads from the provided [Lexer].
    pub fn new(lexer: &'a mut Lexer<'a>) -> ASTParser<'a> {
        return ASTParser { lexer };
    }

    /// Returns the next AST node at the current position in the source code.
    pub fn next_statement(&mut self) -> Result<Statement, Error> {
        // The start of a variable declaration must always start with the `let` keyword.
        let let_token = self.expect_token(TokenKind::Keyword(Keyword::Let))?;

        // The next token must be an identifier.
        let (identifier, _) = self.expect_identifier()?;

        // The next token must be an equals.
        self.expect_token(TokenKind::Equals)?;

        // And finally, an expression must be provided for the initial value.
        let value = self.next_expression()?;

        Ok(Statement {
            span: SourceSpan::between(&let_token.span, &value.span),
            kind: VariableDeclaration::new(identifier, value).into(),
        })
    }

    fn next_expression(&mut self) -> Result<Expression, Error> {
        // The only expression type that is supported is the integer literal.
        let token = self.lexer.next_token()?;

        let integer_literal = match token.kind {
            TokenKind::IntegerLiteral(literal) => literal,
            _ => {
                return Error {
                    kind: ASTErrorKind::UnexpectedEndOfFile.into(),
                    span: token.span,
                }
                .into();
            }
        };

        Ok(Expression {
            kind: ExpressionKind::IntegerLiteral(integer_literal),
            span: token.span,
        })
    }

    /// Expects a certain [TokenKind] to be produced by the lexer, returning an [Err] if a different token was returned.
    fn expect_token(&mut self, kind: TokenKind) -> Result<Token, Error> {
        let token = self.next_token()?;

        // If the token's kind does not match, we can return an error.
        if token.kind != kind {
            return Error {
                kind: ASTErrorKind::UnexpectedToken {
                    expected: kind,
                    received: token.kind,
                }
                .into(),
                span: token.span,
            }
            .into();
        }

        Ok(token)
    }

    /// Expects an identifier token to be produced by the lexer, returning an [Err] if a different token was returned.
    fn expect_identifier(&mut self) -> Result<(String, Token), Error> {
        let token = self.next_token()?;

        match &token.kind {
            TokenKind::Identifier(identifier) => Ok((identifier.into(), token)),

            _ => Error {
                kind: ASTErrorKind::UnexpectedToken {
                    expected: TokenKind::Identifier("".into()),
                    received: token.kind,
                }
                .into(),
                span: token.span,
            }
            .into(),
        }
    }

    /// Returns the next token from the lexer that is not considered to be a whitespace token.
    fn next_token(&mut self) -> Result<Token, Error> {
        // We would like to consume tokens until we find a token that is not whitespace.
        loop {
            let next_token = self.lexer.next_token()?;

            // If the token is the EOF token, then we can return an error.
            if next_token.kind == TokenKind::EOF {
                return Error {
                    kind: ASTErrorKind::UnexpectedEndOfFile.into(),
                    span: next_token.span,
                }
                .into();
            }

            // Otherwise, we can continue searching for tokens if this token can be considered as whitespace.
            if next_token.is_considered_whitespace() {
                continue;
            }

            // We have found a non-whitespace token!
            return Ok(next_token);
        }
    }
}

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.\
    use super::*;
    use crate::core::source_span::SourceSpan;

    #[test]
    fn test_variable_declaration() {
        let mut lexer = Lexer::new("let identifier = 123456;");
        let mut ast_parser = ASTParser::new(&mut lexer);

        assert_eq!(
            ast_parser.next_statement().expect("next_statement should not fail!"),
            Statement {
                kind: VariableDeclaration::new(
                    "identifier".to_string(),
                    Expression {
                        kind: ExpressionKind::IntegerLiteral(123456),
                        span: SourceSpan { start: 16, end: 23 }
                    }
                )
                .into(),
                span: SourceSpan { start: 0, end: 23 }
            }
        )
    }
}
