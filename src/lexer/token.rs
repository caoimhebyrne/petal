use crate::core::location::Location;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Keyword {
    Func,
    Return,
    Extern,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum TokenKind {
    IntegerLiteral(u64), // An integer literal token.
    Identifier(String),  // An identifier.
    Keyword(Keyword),    // A keyword.

    Plus,             // +
    Minus,            // -
    Asterisk,         // *
    Slash,            // /
    Equals,           // =
    Colon,            // :
    Semicolon,        // ;
    OpenParenthesis,  // (
    CloseParenthesis, // )
    OpenBrace,        // {
    CloseBrace,       // }
    GreaterThan,      // >
    Comma,            // ,
}

#[derive(Debug, Clone)]
pub struct Token {
    pub kind: TokenKind,
    pub location: Location,
}

impl Token {
    pub fn new(kind: TokenKind, location: Location) -> Self {
        Self { kind, location }
    }
}
