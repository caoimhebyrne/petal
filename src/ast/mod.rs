use crate::{
    ast::{
        error::ASTErrorKind,
        expression::{Expression, ExpressionKind},
        statement::{Statement, VariableDeclaration},
    },
    core::{
        error::{Error, Result},
        source_span::SourceSpan,
        string_intern::{StringInternPool, StringReference},
    },
    lexer::{
        stream::TokenStream,
        token::{Keyword, Token, TokenKind},
    },
};

pub mod error;
pub mod expression;
pub mod statement;

/// Converts tokens from a [Lexer] into an Abstract Syntax Tree.
pub struct ASTParser<'a> {
    /// The string intern pool to read strings from.
    #[allow(dead_code)]
    string_intern_pool: &'a mut dyn StringInternPool,

    /// The token stream to read tokens from.
    token_stream: TokenStream,
}

impl<'a> ASTParser<'a> {
    /// Creates a new [ASTParser] which reads from the provided [Lexer].
    pub fn new(string_intern_pool: &'a mut dyn StringInternPool, token_stream: TokenStream) -> Self {
        return ASTParser {
            string_intern_pool,
            token_stream,
        };
    }

    /// Returns the next AST node at the current position in the source code.
    pub fn next_statement(&mut self) -> Result<Statement> {
        // The start of a variable declaration must always start with the `let` keyword.
        let let_token = self.expect_token(TokenKind::Keyword(Keyword::Let))?;

        // The next token must be an identifier.
        let (identifier_reference, _) = self.expect_identifier()?;

        // The next token must be an equals.
        self.expect_token(TokenKind::Equals)?;

        // And finally, an expression must be provided for the initial value.
        let value = self.next_expression()?;

        Ok(Statement {
            span: SourceSpan::between(&let_token.span, &value.span),
            kind: VariableDeclaration::new(identifier_reference, value).into(),
        })
    }

    fn next_expression(&mut self) -> Result<Expression> {
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
    fn expect_token(&mut self, kind: TokenKind) -> Result<Token> {
        let token = self
            .token_stream
            .next_non_whitespace()
            .ok_or_else(|| ASTErrorKind::unexpected_end_of_file())?;

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

        Ok(*token)
    }

    /// Expects an identifier token to be produced by the lexer, returning an [Err] if a different token was returned.
    fn expect_identifier(&mut self) -> Result<(StringReference, Token)> {
        let token = self
            .token_stream
            .next_non_whitespace()
            .ok_or_else(|| ASTErrorKind::unexpected_end_of_file())?;

        match token.kind {
            TokenKind::Identifier(reference) => Ok((reference, *token)),

            _ => Error {
                kind: ASTErrorKind::ExpectedIdentifier { received: token.kind }.into(),
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
    use crate::{
        core::{source_span::SourceSpan, string_intern::StringInternPoolImpl},
        lexer::Lexer,
    };

    #[test]
    fn test_variable_declaration() {
        let mut string_intern_pool = StringInternPoolImpl::new();
        let identifier_reference = StringReference(0);

        let mut lexer = Lexer::new(&mut string_intern_pool, "let identifier = 123456;");
        let token_stream = lexer.get_stream().expect("get_stream should not fail");

        let mut ast_parser = ASTParser::new(&mut string_intern_pool, token_stream);

        assert_eq!(
            ast_parser.next_statement().expect("next_statement should not fail!"),
            Statement {
                kind: VariableDeclaration::new(
                    identifier_reference,
                    Expression {
                        kind: ExpressionKind::IntegerLiteral(123456),
                        span: SourceSpan { start: 17, end: 23 }
                    }
                )
                .into(),
                span: SourceSpan { start: 0, end: 23 }
            }
        );

        assert_eq!(
            string_intern_pool.resolve_reference(&identifier_reference),
            Some("identifier")
        );
    }
}
