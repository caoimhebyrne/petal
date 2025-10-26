use std::fmt::Display;

use petal_core::{
    error::{Error, ErrorKind},
    source_span::SourceSpan,
};
use petal_lexer::token::{Token, TokenKind};

/// Represents the different kinds of errors that can be returned when parsing an AST.
#[derive(Debug, Clone, PartialEq)]
pub enum ASTErrorKind {
    /// The end of the file was reached when it was not expected.
    UnexpectedEndOfFile,

    /// A certain token was expected at a point in the source code, but a different token was found.
    ExpectedToken { expected: TokenKind, received: TokenKind },

    /// An identifier was expected at a point in the source code, but a different token was found.
    ExpectedIdentifier { received: TokenKind },

    /// A statement was expected, but a different token kind was received.
    ExpectedStatement { received: TokenKind },

    /// An expression was expected, but a different token kind was received.
    ExpectedExpression { received: TokenKind },
}

impl ASTErrorKind {
    pub fn unexpected_end_of_file() -> Error {
        Error::new(ASTErrorKind::UnexpectedEndOfFile, SourceSpan { start: 0, end: 0 })
    }

    pub fn expected_token(expected: TokenKind, received: &Token) -> Error {
        Error::new(
            ASTErrorKind::ExpectedToken {
                expected,
                received: received.kind,
            },
            received.span,
        )
    }

    pub fn expected_identifier(received: &Token) -> Error {
        Error::new(
            ASTErrorKind::ExpectedIdentifier {
                received: received.kind,
            },
            received.span,
        )
    }

    pub fn expected_statement(received: &Token) -> Error {
        Error::new(
            ASTErrorKind::ExpectedStatement {
                received: received.kind,
            },
            received.span,
        )
    }

    pub fn expected_expression(received: &Token) -> Error {
        Error::new(
            ASTErrorKind::ExpectedExpression {
                received: received.kind,
            },
            received.span,
        )
    }
}

impl ErrorKind for ASTErrorKind {}

impl Display for ASTErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ASTErrorKind::UnexpectedEndOfFile => write!(f, "Unexpected end-of-file"),

            ASTErrorKind::ExpectedToken { expected, received } => {
                write!(
                    f,
                    "Expected token '{:?}', but received token '{:?}'",
                    expected, received
                )
            }

            ASTErrorKind::ExpectedIdentifier { received } => {
                write!(f, "Expected an identifier, but received token '{:?}'", received)
            }

            ASTErrorKind::ExpectedStatement { received } => {
                write!(f, "Expected any statement, but received token '{:?}'", received)
            }

            ASTErrorKind::ExpectedExpression { received } => {
                write!(f, "Expected an expression, but received token '{:?}'", received)
            }
        }
    }
}
