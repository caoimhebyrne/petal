use crate::core::span::Span;

/// A single token parsed by the [`Lexer`].
#[derive(Debug, Clone, PartialEq)]
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
#[derive(Debug, PartialEq, Clone)]
pub enum TokenKind {
    /// An opening parenthesis: (
    OpenParen,

    /// A closing parenthesis: (
    CloseParen,

    /// An equals: =
    Equals,

    /// A double-equals: ==
    DoubleEquals,

    /// A not-equals: !=
    NotEquals,

    /// A forward slash: /
    ForwardSlash,

    /// A hyphen: -
    Hyphen,

    /// A left angle bracket: <
    LeftAngleBracket,

    /// A right angle bracket: >
    RightAngleBracket,

    /// An opening brace: {
    OpenBrace,

    /// A closing brace: }
    CloseBrace,

    /// A semicolon: ;
    Semicolon,

    /// A colon: :
    Colon,

    /// Two colons: ::
    DoubleColon,

    /// A comma: ,
    Comma,

    /// A plus: +
    Plus,

    /// An asterisk: *
    Asterisk,

    /// A tilda: ~
    Tilda,

    /// An exclamation mark: !
    ExclamationMark,

    /// An ampersand: &
    Ampersand,

    /// At: @
    At,

    /// Period: .
    Period,

    /// Question mark: ?
    QuestionMark,

    /// A string literal.
    String(String),

    /// An identifier.
    Identifier(String),

    /// A keyword.
    Keyword(Keyword),

    /// A floating-point number literal token.
    Number(f64),
}

/// A reserved keyword in the Petal programming language.
#[derive(Debug, PartialEq, Clone)]
pub enum Keyword {
    /// The `func` keyword.
    Func,

    /// The `return` keyword.
    Return,

    /// The true keyword (boolean literal).
    True,

    /// The false keyword (boolean literal).
    False,

    /// An if expression.
    If,

    /// An else expression.
    Else,

    /// An import statement.
    Import,

    /// The public access modifier.
    Public,

    /// A type declaration.
    Type,

    /// A structure type declaration.
    Struct,

    /// A namespace declaration.
    Namespace,

    /// An external function declaration.
    Extern,
}

impl Keyword {
    /// Attempts to create a [`Keyword`] based on an input string.
    /// If the input string does not correspond to a keyword, then [`None`] will be returned.
    pub fn from(string: &str) -> Option<Keyword> {
        let keyword = match string {
            "func" => Keyword::Func,
            "return" => Keyword::Return,
            "true" => Keyword::True,
            "false" => Keyword::False,
            "if" => Keyword::If,
            "else" => Keyword::Else,
            "import" => Keyword::Import,
            "public" => Keyword::Public,
            "type" => Keyword::Type,
            "struct" => Keyword::Struct,
            "namespace" => Keyword::Namespace,
            "extern" => Keyword::Extern,

            _ => return None,
        };

        Some(keyword)
    }
}
