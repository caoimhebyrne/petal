use std::fmt::Display;

use crate::{
    core::{error::Error, source_span::SourceSpan},
    lexer::token::TokenKind,
};

/// Represents the different kinds of errors that can be returned when parsing an AST.
#[derive(Debug, Clone, PartialEq)]
pub enum ASTErrorKind {
    /// The end of the file was reached when it was not expected.
    UnexpectedEndOfFile,

    /// A certain token was expected at a point in the source code, but a different token was found.
    UnexpectedToken { expected: TokenKind, received: TokenKind },
}

impl ASTErrorKind {
    pub fn unexpected_end_of_file() -> Error {
        Error {
            kind: ASTErrorKind::UnexpectedEndOfFile.into(),
            span: SourceSpan { start: 0, end: 0 },
        }
    }
}

impl Display for ASTErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ASTErrorKind::UnexpectedEndOfFile => write!(f, "Unexpected end-of-file"),
            ASTErrorKind::UnexpectedToken { expected, received } => {
                write!(
                    f,
                    "Expected token '{:?}', but received token '{:?}'",
                    expected, received
                )
            }
        }
    }
}
