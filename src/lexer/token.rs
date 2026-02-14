use crate::core::span::Span;

/// A single token parsed by the [`Lexer`].
#[derive(Debug, PartialEq)]
pub struct Token {
    /// The kind of token that this is.
    pub kind: TokenKind,

    /// The location within the original source file that this token occurred at.
    pub span: Span,
}

impl Token {
    /// Creates a new [`Token`].
    pub fn new(kind: TokenKind, span: Span) -> Self {
        Token { kind, span }
    }
}

/// The different kinds of tokens that exist.
#[derive(Debug, PartialEq)]
pub enum TokenKind {
    /// An opening parenthesis: (
    OpenParen,

    /// A closing parenthesis: (
    CloseParen,

    /// An equals: =
    Equals,

    /// A forward slash: /
    ForwardSlash,

    /// A hyphen: -
    Hyphen,

    /// A right angle bracket: >
    RightAngleBracket,

    /// An opening brace: {
    OpenBrace,

    /// A closing brace: }
    CloseBrace,

    /// A semicolon: ;
    Semicolon,

    /// An identifier.
    Identifier(String),

    /// A keyword.
    Keyword(Keyword),

    /// A floating-point number literal token.
    Number(f64),
}

/// A reserved keyword in the Petal programming language.
#[derive(Debug, PartialEq)]
pub enum Keyword {
    /// The `func` keyword.
    Func,

    /// The `return` keyword.
    Return,
}

impl Keyword {
    /// Attempts to create a [`Keyword`] based on an input string.
    /// If the input string does not correspond to a keyword, then [`None`] will be returned.
    pub fn from(string: &str) -> Option<Keyword> {
        let keyword = match string {
            "func" => Keyword::Func,
            "return" => Keyword::Return,

            _ => return None,
        };

        Some(keyword)
    }
}
