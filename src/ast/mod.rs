use crate::{
    ast::{
        error::ASTErrorKind,
        expression::{Expression, ExpressionKind},
        statement::{Statement, VariableDeclaration},
    },
    core::{error::Error, source_span::SourceSpan},
    lexer::{
        stream::TokenStream,
        token::{Keyword, Token, TokenKind},
    },
};

pub mod error;
pub mod expression;
pub mod statement;

/// Converts tokens from a [Lexer] into an Abstract Syntax Tree.
pub struct ASTParser {
    /// The token stream to read tokens from.
    token_stream: TokenStream,
}

impl ASTParser {
    /// Creates a new [ASTParser] which reads from the provided [Lexer].
    pub fn new(token_stream: TokenStream) -> ASTParser {
        return ASTParser { token_stream };
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
        let token = self
            .token_stream
            .next_non_whitespace()
            .ok_or_else(|| ASTErrorKind::unexpected_end_of_file())?;

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
        let token = self
            .token_stream
            .next_non_whitespace()
            .ok_or_else(|| ASTErrorKind::unexpected_end_of_file())?;

        // If the token's kind does not match, we can return an error.
        if token.kind != kind {
            return Error {
                kind: ASTErrorKind::UnexpectedToken {
                    expected: kind,
                    received: token.kind.clone(),
                }
                .into(),
                span: token.span,
            }
            .into();
        }

        Ok(token.clone())
    }

    /// Expects an identifier token to be produced by the lexer, returning an [Err] if a different token was returned.
    fn expect_identifier(&mut self) -> Result<(String, Token), Error> {
        let token = self
            .token_stream
            .next_non_whitespace()
            .ok_or_else(|| ASTErrorKind::unexpected_end_of_file())?;

        match &token.kind {
            TokenKind::Identifier(identifier) => Ok((identifier.into(), token.clone())),

            _ => Error {
                kind: ASTErrorKind::UnexpectedToken {
                    expected: TokenKind::Identifier("".into()),
                    received: token.kind.clone(),
                }
                .into(),
                span: token.span,
            }
            .into(),
        }
    }
}

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.\
    use super::*;
    use crate::{core::source_span::SourceSpan, lexer::Lexer};

    #[test]
    fn test_variable_declaration() {
        let mut lexer = Lexer::new("let identifier = 123456;");

        let token_stream = lexer.get_stream().expect("get_stream should not fail");

        let mut ast_parser = ASTParser::new(token_stream);

        assert_eq!(
            ast_parser.next_statement().expect("next_statement should not fail!"),
            Statement {
                kind: VariableDeclaration::new(
                    "identifier".to_string(),
                    Expression {
                        kind: ExpressionKind::IntegerLiteral(123456),
                        span: SourceSpan { start: 17, end: 23 }
                    }
                )
                .into(),
                span: SourceSpan { start: 0, end: 23 }
            }
        )
    }
}
