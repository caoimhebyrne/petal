use std::fmt::Display;

use crate::{
    core::{error::Error, source_span::SourceSpan},
    lexer::token::{Token, TokenKind},
};

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
}

impl ASTErrorKind {
    pub fn unexpected_end_of_file() -> Error {
        Error {
            kind: ASTErrorKind::UnexpectedEndOfFile.into(),
            span: SourceSpan { start: 0, end: 0 },
        }
    }

    pub fn expected_token(expected: TokenKind, received: &Token) -> Error {
        Error {
            kind: ASTErrorKind::ExpectedToken {
                expected,
                received: received.kind,
            }
            .into(),
            span: received.span,
        }
    }

    pub fn expected_identifier(received: &Token) -> Error {
        Error {
            kind: ASTErrorKind::ExpectedIdentifier {
                received: received.kind,
            }
            .into(),
            span: received.span,
        }
    }

    pub fn expected_statement(received: &Token) -> Error {
        Error {
            kind: ASTErrorKind::ExpectedStatement {
                received: received.kind,
            }
            .into(),
            span: received.span,
        }
    }
}

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
        }
    }
}
