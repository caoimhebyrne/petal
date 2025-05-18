#[derive(Debug, Clone)]
pub enum TokenKind {
    IntegerLiteral(u64), // An integer literal token.
    Identifier(String),  // An identifier.
    Keyword(String),     // A keyword.

    Plus,             // +
    Minus,            // -
    Asterisk,         // *
    Slash,            // /
    Equals,           // =
    Semicolon,        // ;
    OpenParenthesis,  // (
    CloseParenthesis, // )
    OpenBrace,        // {
    CloseBrace,       // }
    GreaterThan,      // >
}

#[derive(Debug, Clone)]
pub struct Token {
    pub kind: TokenKind,
}

impl Token {
    pub fn new(kind: TokenKind) -> Token {
        Token { kind }
    }
}
