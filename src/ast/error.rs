use std::fmt::Display;

use crate::{
    core::{
        error::Error,
        span::Span,
    },
    lexer::token::TokenKind,
};

/// An AST error.
#[derive(Debug, PartialEq)]
pub struct ASTError {
    /// The kind of AST error that this is.
    pub kind: ASTErrorKind,

    /// The [`Span`] that the error occurred at.
    pub span: Span,
}

/// The different kinds of [`ASTError`]s that exist.
#[derive(Debug, PartialEq)]
pub enum ASTErrorKind {
    ExpectedToken { expected: TokenKind, got: TokenKind },
    ExpectedIdentifier,
    UnexpectedToken(TokenKind),
    UnexpectedEndOfFile,
}

impl ASTErrorKind {
    /// Returns an [ASTError] from this [ASTErrorKind] at the provided [Span].
    pub fn at(self, span: Span) -> ASTError {
        ASTError { kind: self, span }
    }
}

impl ASTError {
    /// Creates a new [`ASTError`].
    pub fn new(kind: ASTErrorKind, span: Span) -> Self {
        ASTError { kind, span }
    }
}

impl Display for ASTErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ASTErrorKind::ExpectedToken { expected, got } => {
                write!(f, "Expected token '{:?}' but got '{:?}'", expected, got)
            }
            ASTErrorKind::ExpectedIdentifier => write!(f, "Expected any identifier"),
            ASTErrorKind::UnexpectedToken(token) => write!(f, "Unexpected token: '{:?}'", token),
            ASTErrorKind::UnexpectedEndOfFile => write!(f, "Unexpected end-of-file"),
        }
    }
}

impl Error for ASTError {
    fn span(&self) -> Option<Span> {
        Some(self.span)
    }
}

impl Display for ASTError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.kind.fmt(f)
    }
}
