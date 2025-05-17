#[derive(Debug, Copy, Clone)]
pub enum TokenKind {
    IntegerLiteral(u64), // An integer literal token.

    Plus,     // +
    Minus,    // -
    Asterisk, // *
    Slash,    // /
}

#[derive(Debug, Copy, Clone)]
pub struct Token {
    pub kind: TokenKind,
}

impl Token {
    pub fn new(kind: TokenKind) -> Token {
        Token { kind }
    }
}
