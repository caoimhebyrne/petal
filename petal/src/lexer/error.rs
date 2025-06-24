use crate::core::location::Location;
use std::fmt::Display;

#[derive(Debug, Clone)]
pub enum LexerErrorKind {
    UnexpectedCharacter(char),
    UnexpectedEndOfFile,
    InvalidIntegerLiteral(String),
}

#[derive(Debug, Clone)]
pub struct LexerError {
    pub kind: LexerErrorKind,
    pub location: Location,
}

impl LexerError {
    pub fn unexpected_character(character: char, location: Location) -> LexerError {
        LexerError {
            kind: LexerErrorKind::UnexpectedCharacter(character),
            location,
        }
    }

    pub fn unexpected_end_of_file(location: Location) -> LexerError {
        LexerError {
            kind: LexerErrorKind::UnexpectedEndOfFile,
            location,
        }
    }

    pub fn invalid_integer_literal(value: String, location: Location) -> LexerError {
        LexerError {
            kind: LexerErrorKind::InvalidIntegerLiteral(value),
            location,
        }
    }
}

impl Display for LexerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.kind {
            LexerErrorKind::UnexpectedCharacter(character) => {
                write!(f, "Unexpected character: '{}'", character)
            }

            LexerErrorKind::UnexpectedEndOfFile => write!(f, "Unexpected end-of-file"),

            LexerErrorKind::InvalidIntegerLiteral(value) => {
                write!(f, "Invalid integer literal: '{}'", value)
            }
        }
    }
}
