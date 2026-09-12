use petal_span::Span;

/// A token parsed from source code.
///
/// Tokens do not contain any data for literals (i.e. strings, numbers). The caller is expected to extract them (and
/// parse them) from the source string using [`Self::span`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Token {
    /// The kind of token that this is.
    pub kind: TokenKind,

    /// The span within the source code that this token was parsed at.
    pub span: Span,
}

impl Token {
    /// Create a new [`Token`] from a [`TokenKind`] and [`Span`].
    pub const fn new(kind: TokenKind, span: Span) -> Self {
        Self { kind, span }
    }
}

/// The different kinds of [`Token`]s that can be parsed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenKind {
    Identifier,
    Number { is_floating_point: bool },

    OpenBrace,
    CloseBrace,
    Colon,
    Semicolon,
    Comma,
    Hyphen,
    Slash,

    Arrow,

    Comment,
    Unknown,
}
