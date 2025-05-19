use crate::{
    core::location::Location,
    lexer::token::{Token, TokenKind},
};
use std::fmt::Display;

#[derive(Debug, Clone)]
pub enum ASTErrorKind {
    ExpectedToken {
        expected: TokenKind,
        received: Option<Token>,
    },

    UnexpectedToken(Token),
    UnexpectedEndOfFile,
}

#[derive(Debug, Clone)]
pub struct ASTError {
    pub kind: ASTErrorKind,
    pub location: Option<Location>,
}

impl ASTError {
    pub fn expected_token(expected: TokenKind, received: Option<Token>) -> ASTError {
        ASTError {
            kind: ASTErrorKind::ExpectedToken {
                expected,
                received: received.clone(),
            },
            location: received.map(|it| it.location),
        }
    }

    pub fn unexpected_token(token: Token) -> ASTError {
        ASTError {
            kind: ASTErrorKind::UnexpectedToken(token.clone()),
            location: Some(token.location),
        }
    }

    pub fn unexpected_end_of_file() -> ASTError {
        ASTError {
            kind: ASTErrorKind::UnexpectedEndOfFile,
            location: None,
        }
    }
}

impl Display for ASTError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.kind {
            ASTErrorKind::ExpectedToken { expected, received } => {
                if let Some(received_token) = received {
                    write!(
                        f,
                        "Expected token '{:?}', but received '{:?}'",
                        expected, received_token.kind
                    )
                } else {
                    write!(f, "Expected token '{:?}', but received end-of-file", expected)
                }
            }

            ASTErrorKind::UnexpectedToken(token) => {
                write!(f, "Unexpected token: '{:?}'", token.kind)
            }

            ASTErrorKind::UnexpectedEndOfFile => {
                write!(f, "Unexpected end-of-file")
            }
        }
    }
}
