use std::fmt::Display;

use petal_core::error::ErrorKind;

/// Represents the different errors that can be returned by a [Lexer].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LexerErrorKind {
    /// An unexpected character was found in the source code.
    UnexpectedCharacter(char),

    /// An invalid integer literal was found in the source code.
    InvalidIntegerLiteral,
}

impl Display for LexerErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LexerErrorKind::UnexpectedCharacter(character) => {
                write!(f, "Unexpected character: {}", character)
            }

            LexerErrorKind::InvalidIntegerLiteral => {
                write!(f, "Invalid integer literal")
            }
        }
    }
}

impl ErrorKind for LexerErrorKind {}
