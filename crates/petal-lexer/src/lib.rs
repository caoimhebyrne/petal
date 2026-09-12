mod cursor;
mod token;

use petal_span::Span;
pub use token::*;

use crate::cursor::Cursor;

struct Lexer<'a> {
    /// The [`Cursor`] to consume characters from.
    cursor: Cursor<'a>,
}

impl<'a> Lexer<'a> {
    /// Create a new [`Lexer`] from a string slice.
    pub fn new(string: &'a str) -> Self {
        Self {
            cursor: Cursor::new(string),
        }
    }

    /// Return the token at the lexer's current position, advancing its cursor.
    pub fn next_token(&mut self) -> Option<Token> {
        // Before we parse a token, we must consume any whitespace, otherwise the offsets will be all wrong.
        self.cursor.consume_while(char::is_whitespace);

        let start = self.cursor.offset();
        let kind = self.next_token_kind()?;
        let end = self.cursor.offset();

        Some(Token::new(
            kind,
            Span::new(
                u32::try_from(start).expect("source was over 4 GiB"),
                u32::try_from(end).expect("source was over 4 GiB"),
            ),
        ))
    }

    /// Return the token kind at the lexer's current position, advancing its cursor.
    fn next_token_kind(&mut self) -> Option<TokenKind> {
        let kind = match self.cursor.consume()? {
            '{' => TokenKind::OpenBrace,
            '}' => TokenKind::CloseBrace,
            ':' => TokenKind::Colon,
            ';' => TokenKind::Semicolon,
            ',' => TokenKind::Comma,

            '-' => {
                if self.cursor.consume_if(|it| it == '>') {
                    TokenKind::Arrow
                } else {
                    TokenKind::Hyphen
                }
            }

            '/' => {
                if self.cursor.consume_if(|it| it == '/') {
                    self.cursor.consume_while(|it| it != '\n');
                    TokenKind::Comment
                } else {
                    TokenKind::Slash
                }
            }

            char if Self::is_identifier_start(char) => {
                self.cursor.consume_while(Self::is_identifier);
                TokenKind::Identifier
            }

            char if char.is_numeric() => {
                self.cursor.consume_while(char::is_numeric);

                let is_floating_point = self.cursor.consume_if(|it| it == '.');
                if is_floating_point {
                    self.cursor.consume_while(char::is_numeric);
                }

                TokenKind::Number {
                    float: is_floating_point,
                }
            }

            _ => TokenKind::Unknown,
        };

        Some(kind)
    }

    /// Return whether a character is valid as the start of an identifier.
    fn is_identifier_start(character: char) -> bool {
        character.is_alphabetic() || character == '_'
    }

    /// Return whether a character is valid as part of an identifier (not the start)
    fn is_identifier(character: char) -> bool {
        character.is_alphanumeric() || character == '_'
    }
}

/// Parse tokens from the provided string literal.
pub fn tokenize(string: &str) -> impl Iterator<Item = Token> + '_ {
    let mut lexer = Lexer::new(string);
    std::iter::from_fn(move || lexer.next_token())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn token_kinds(string: &str) -> Vec<TokenKind> {
        tokenize(string).map(|it| it.kind).collect()
    }

    #[test]
    fn parses_trivial_tokens() {
        assert_eq!(token_kinds("{}-"), vec![
            TokenKind::OpenBrace,
            TokenKind::CloseBrace,
            TokenKind::Hyphen
        ]);
    }

    #[test]
    fn parses_comment() {
        assert_eq!(token_kinds("// Hello, world!\nfoo"), vec![
            TokenKind::Comment,
            TokenKind::Identifier
        ]);
    }

    #[test]
    fn parses_combination_tokens() {
        assert_eq!(token_kinds("->"), vec![TokenKind::Arrow]);
    }

    #[test]
    fn parses_identifier_token() {
        assert_eq!(tokenize("hello").collect::<Vec<_>>(), vec![Token::new(
            TokenKind::Identifier,
            Span::new(0, 5)
        )]);
    }

    #[test]
    fn parses_integer_token() {
        assert_eq!(token_kinds("123456"), vec![TokenKind::Number { float: false }]);
    }

    #[test]
    fn parses_float_token() {
        assert_eq!(tokenize("123456.789").collect::<Vec<_>>(), vec![Token::new(
            TokenKind::Number { float: true },
            Span::new(0, 10)
        )]);
    }

    #[test]
    fn identifier_span_does_not_include_whitespace() {
        let source = "hello world";
        let tokens = tokenize(source).collect::<Vec<_>>();
        let hello = tokens[0];
        let world = tokens[1];

        assert_eq!(hello.span.slice(source), "hello");
        assert_eq!(world.span.slice(source), "world");
    }
}
