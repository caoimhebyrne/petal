use std::fmt::Display;

#[derive(Debug, Clone)]
pub enum LexerErrorKind {
    UnexpectedCharacter(char),

    InvalidIntegerLiteral(String),
}

#[derive(Debug, Clone)]
pub struct LexerError {
    kind: LexerErrorKind,
}

impl LexerError {
    pub fn unexpected_character(character: char) -> LexerError {
        LexerError {
            kind: LexerErrorKind::UnexpectedCharacter(character),
        }
    }

    pub fn invalid_integer_literal(value: String) -> LexerError {
        LexerError {
            kind: LexerErrorKind::InvalidIntegerLiteral(value),
        }
    }
}

impl Display for LexerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.kind {
            LexerErrorKind::UnexpectedCharacter(character) => {
                write!(f, "Unexpected character: '{}'", character)
            }

            LexerErrorKind::InvalidIntegerLiteral(value) => {
                write!(f, "Invalid integer literal: '{}'", value)
            }
        }
    }
}
