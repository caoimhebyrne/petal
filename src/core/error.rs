use crate::{
    ast::error::ASTErrorKind, core::source_span::SourceSpan, lexer::error::LexerErrorKind,
};
use std::fmt::{Debug, Display};

/// Represents the different kinds of errors that can occur during compilation.
#[derive(Debug, Clone, PartialEq)]
pub enum ErrorKind {
    AST(ASTErrorKind),
    Lexer(LexerErrorKind),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Error {
    /// The kind of error that this is.
    pub kind: ErrorKind,

    /// The location in the source that the error occurred at.
    pub span: SourceSpan,
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.kind)
    }
}

impl Display for ErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ErrorKind::AST(kind) => write!(f, "{}", kind),
            ErrorKind::Lexer(kind) => write!(f, "{}", kind),
        }
    }
}

/// Allows .into() to be called on an `Error to convert it into a `Result<T, Error>`.
impl<T> From<Error> for Result<T, Error> {
    fn from(value: Error) -> Result<T, Error> {
        return Err(value);
    }
}

/// Allows `.into()` to be called on a `ASTErrorKind` to turn it into an `ErrorKind`.
impl From<ASTErrorKind> for ErrorKind {
    fn from(value: ASTErrorKind) -> ErrorKind {
        ErrorKind::AST(value)
    }
}

/// Allows `.into()` to be called on a `LexerErrorKind` to turn it into an `ErrorKind`.
impl From<LexerErrorKind> for ErrorKind {
    fn from(value: LexerErrorKind) -> ErrorKind {
        ErrorKind::Lexer(value)
    }
}
