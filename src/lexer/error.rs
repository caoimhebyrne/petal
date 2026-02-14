use std::fmt::Display;

use crate::core::{error::Error, span::Span};

/// A lexer error.
#[derive(Debug, PartialEq)]
pub struct LexerError {
    /// The kind of lexer error that this is.
    pub kind: LexerErrorKind,

    /// The [`Span`] that the error occurred at.
    pub span: Span,
}

/// The different kinds of [`LexerError`]s that exist.
#[derive(Debug, PartialEq)]
pub enum LexerErrorKind {
    /// A number literal is invalid.
    InvalidNumberLiteral(String),

    /// A character was reached that is unrecognized.
    UnrecognizedCharacter(char),
}

impl LexerError {
    /// Creates a new [`LexerError`].
    pub fn new(kind: LexerErrorKind, span: Span) -> Self {
        LexerError { kind, span }
    }
}

impl Error for LexerError {
    fn span(&self) -> Option<Span> {
        Some(self.span)
    }
}

impl Display for LexerErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LexerErrorKind::InvalidNumberLiteral(value) => write!(f, "Invalid number literal: '{}'", value),
            LexerErrorKind::UnrecognizedCharacter(char) => write!(f, "Unrecognized character: '{}'", char),
        }
    }
}

impl Display for LexerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.kind.fmt(f)
    }
}
