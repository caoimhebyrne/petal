/// A token is a small piece of information parsed from the source code.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Token {
    /// The type of token that this is.
    pub kind: TokenKind,

    /// The start offset of the token in the source code.
    pub start: usize,

    /// The end offset of the token in the source code.
    pub end: usize,
}

/// Represents the different kinds of tokens that are available.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum TokenKind {
    /// The end-of-file token.
    EOF,

    /// An integer literal.
    IntegerLiteral(u64),

    /// An identifier.
    /// FIXME: A `String` is not optimal here. It would make sense to implement some string intering in the future:
    /// https://en.wikipedia.org/wiki/String_interning
    Identifier(String),

    /// A keyword.
    Keyword(Keyword),

    /// =
    Equals,

    // ;
    Semicolon,
}

/// Represents the different kinds of keywords that are available.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Keyword {
    /// Variable definition.
    Let,

    /// Function definition.
    Func,
}
