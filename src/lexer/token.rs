use crate::core::{source_span::SourceSpan, string_intern::StringReference};

/// A token is a small piece of information parsed from the source code.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Token {
    /// The type of token that this is.
    pub kind: TokenKind,

    /// The span in the source code that this token is in.
    pub span: SourceSpan,
}

impl Token {
    /// Returns whether this token is considered to be whitespace. If true, most parsers can ignore it.
    pub fn is_considered_whitespace(&self) -> bool {
        match self.kind {
            TokenKind::Comment(_) => true,
            _ => false,
        }
    }
}

/// Represents the different kinds of tokens that are available.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenKind {
    /// The end-of-file token.
    EOF,

    /// An integer literal.
    IntegerLiteral(u64),

    /// An identifier.
    Identifier(StringReference),

    /// A keyword.
    Keyword(Keyword),

    /// =
    Equals,

    // ;
    Semicolon,

    // /
    ForwardSlash,

    // (
    LeftParenthesis,

    // (
    RightParenthesis,

    // {
    LeftBrace,

    // }
    RightBrace,

    // -
    Hyphen,

    // >
    RightAngleBracket,

    // This is a token that is ignored by most implementations, but might be useful in the future for some
    // cool tooling.
    Comment(StringReference),
}

/// Represents the different kinds of keywords that are available.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Keyword {
    /// Variable definition.
    Let,

    /// Function definition.
    Func,

    /// A return statement.
    Return,
}
