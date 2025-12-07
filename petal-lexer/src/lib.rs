use std::str::Chars;

use petal_core::{
    error::{Error, Result},
    source_span::SourceSpan,
    string_intern::StringInternPool,
};

use crate::{
    error::LexerErrorKind,
    stream::TokenStream,
    token::{Keyword, Token, TokenKind},
};

type LexerResult<T> = core::result::Result<T, LexerErrorKind>;

pub mod error;
pub mod stream;
pub mod token;

/// The lexer is responsible for taking an input string and producing tokens from that input.
pub struct Lexer<'a> {
    /// The source being parsed.
    source: &'a str,

    /// The remaining characters to be consumed from the source.
    chars: Chars<'a>,

    /// The [StringInternPool] to allocate string instances in.
    string_intern_pool: &'a mut dyn StringInternPool,
}

impl<'a> Lexer<'a> {
    /// Creates a new Lexer instance.
    pub fn new(string_intern_pool: &'a mut dyn StringInternPool, source: &'a str) -> Self {
        return Lexer {
            source,
            string_intern_pool,
            chars: source.chars(),
        };
    }

    /// Returns a stream of tokens that has been read from the source within the [Lexer].
    pub fn get_stream(&mut self) -> Result<TokenStream> {
        let mut tokens = vec![];

        loop {
            let token = self.next_token()?;
            if token.kind == TokenKind::EOF {
                break;
            }

            tokens.push(token);
        }

        Ok(TokenStream::new(tokens))
    }

    /// Returns the token at the lexer's current position.
    fn next_token(&mut self) -> Result<Token> {
        // Before reading the next token, we should attempt to consume any whitespace. This will ensure that our
        // offsets are correct.
        while let Some(character) = self.peek() {
            if Self::is_whitespace(character) {
                self.chars.next();
            } else {
                break;
            }
        }

        let start_offset = self.offset();
        let token_kind_result = self.next_kind();
        let end_offset = self.offset();

        let span = SourceSpan {
            start: start_offset,
            end: end_offset,
        };

        match token_kind_result {
            Ok(kind) => Ok(Token { kind, span }),
            Err(kind) => Err(Error::new(kind, span)),
        }
    }

    /// Returns the next token kind at the lexer's current position.
    fn next_kind(&mut self) -> LexerResult<TokenKind> {
        while let Some(character) = self.chars.next() {
            let kind = match character {
                '=' => TokenKind::Equals,
                ';' => TokenKind::Semicolon,
                '(' => TokenKind::LeftParenthesis,
                ')' => TokenKind::RightParenthesis,
                '{' => TokenKind::LeftBrace,
                '}' => TokenKind::RightBrace,
                '-' => TokenKind::Hyphen,
                '>' => TokenKind::RightAngleBracket,
                ':' => TokenKind::Colon,
                ',' => TokenKind::Comma,
                '+' => TokenKind::Plus,
                '*' => TokenKind::Asterisk,
                '&' => TokenKind::Ampersand,
                '.' => TokenKind::Period,
                '!' => TokenKind::ExclamationMark,

                '/' => return self.parse_forward_slash_or_comment(),
                '"' => return self.parse_string_literal(),

                _ => {
                    // If the character is considered to be whitespace, then continue.
                    if Self::is_whitespace(character) {
                        continue;
                    }

                    // If this is a numeric character, we can attempt to parse an integer literal.
                    if character.is_numeric() {
                        return self.parse_integer_literal(character);
                    }

                    // If this is an alphabetic character, we can attempt to parse an identifier.
                    if character.is_alphabetic() || character == '_' {
                        return self.parse_identifier_or_keyword(character);
                    }

                    // Otherwise, this is an unexpected character.
                    return Err(LexerErrorKind::UnexpectedCharacter(character));
                }
            };

            return Ok(kind);
        }

        // If we break out of the loop, that means the end of the file was reached.
        Ok(TokenKind::EOF)
    }

    /// Returns a TokenKind containing the integer literal at the current position in the source text.
    fn parse_integer_literal(&mut self, first_character: char) -> LexerResult<TokenKind> {
        // The first character must always be an int character.
        let mut characters: Vec<char> = vec![first_character];

        // We can then loop over the characters until we reach a non-numeric character.
        while let Some(character) = self.peek() {
            // If the character is not numeric, then we have consumed everything that we need to for the integer
            // literal.
            if !character.is_numeric() {
                break;
            }

            // We can add the character to the final vec and consume the character.
            characters.push(character);
            self.chars.next();
        }

        // We now have a Vec of characters, we can collect it into a string and attempt to parse a u64.
        let integer_string = characters.iter().collect::<String>();
        let integer = integer_string
            .parse::<u64>()
            .map_err(|_| LexerErrorKind::InvalidIntegerLiteral)?;

        Ok(TokenKind::IntegerLiteral(integer))
    }

    /// Returns a TokenKind containing the identifier or keyword at the current position in the source text.
    fn parse_identifier_or_keyword(&mut self, first_character: char) -> LexerResult<TokenKind> {
        let mut characters: Vec<char> = vec![first_character];

        // We can then loop over the characters until we reach a character that is not supported in an identifier.
        while let Some(character) = self.peek() {
            // If the character is not supported as an identifier character, we can assume that the end of the
            // identifier has been reached.
            if !character.is_alphanumeric() && character != '_' {
                break;
            }

            // Otherwise, we can add the character to the vec and consume it.
            characters.push(character);
            self.chars.next();
        }

        // We now have a vec of characters that we can collect for the identifier.
        let identifier = characters.iter().collect::<String>();
        let kind = if let Some(keyword) = Self::match_keyword(&identifier) {
            TokenKind::Keyword(keyword)
        } else {
            let reference = self.string_intern_pool.intern(&identifier);
            TokenKind::Identifier(reference)
        };

        Ok(kind)
    }

    /// Attempts to parse a forward slash token or a commenet token from the current position in the source text.
    fn parse_forward_slash_or_comment(&mut self) -> LexerResult<TokenKind> {
        // If the next character is not another slash, then this was a single slash token.
        if self.peek() != Some('/') {
            return Ok(TokenKind::ForwardSlash);
        }

        // We can consume the next forward slash, it is part of a comment.
        self.chars.next();

        // Now, we can read all of the comment until a new-line occurs (or the end-of-file).
        let mut characters = vec![];

        while let Some(character) = self.peek() {
            // If the character is a new-line, then the comment is over.
            if character == '\n' || character == '\r' {
                break;
            }

            // Otherwise, it is part of the comment string.
            characters.push(character);
            self.chars.next();
        }

        // If the first character in the comment is a space, and it is immediately followed by another character, then
        // we can trim it. Otherwise, we should retain the weird spacing.
        if characters.len() >= 2 && characters[0] == ' ' && characters[1] != ' ' {
            characters.remove(0);
        }

        let comment = characters.iter().collect::<String>();
        let reference = self.string_intern_pool.intern(&comment);

        Ok(TokenKind::Comment(reference))
    }

    /// Attempts to parse a forward slash token or a commenet token from the current position in the source text.
    fn parse_string_literal(&mut self) -> LexerResult<TokenKind> {
        // Now, we can read all of the comment until a new-line occurs (or the end-of-file).
        let mut characters = vec![];
        let mut reached_closing_quote = false;

        while let Some(character) = self.peek() {
            // If the character is a closing quote, then the comment is over.
            if character == '"' {
                reached_closing_quote = true;
                self.chars.next();

                break;
            }

            self.chars.next();

            // If this character is a `\`, we can attempt to insert a newline, etc. depending on the character after.
            if character == '\\' {
                let next_character = match self.peek() {
                    Some(value) => value,
                    _ => continue,
                };

                match next_character {
                    'n' => characters.push('\n'),
                    't' => characters.push('\t'),

                    _ => {
                        characters.push(character);
                        continue;
                    }
                }

                self.chars.next();
            } else {
                characters.push(character);
            }
        }

        if !reached_closing_quote {
            return Err(LexerErrorKind::UnterminatedStringLiteral);
        }

        // The last character was a closing

        let literal = characters.iter().collect::<String>();
        let reference = self.string_intern_pool.intern(&literal);

        Ok(TokenKind::StringLiteral(reference))
    }

    /// Attempts to match a keyword from the input string, returning [Option::None] if a matching keyword was not found.
    fn match_keyword(string: &str) -> Option<Keyword> {
        let keyword = match string {
            "func" => Keyword::Func,
            "extern" => Keyword::Extern,
            "return" => Keyword::Return,
            "import" => Keyword::Import,
            "type" => Keyword::Type,
            "struct" => Keyword::Struct,
            "true" => Keyword::True,
            "false" => Keyword::False,
            "if" => Keyword::If,
            "else" => Keyword::Else,
            "while" => Keyword::While,
            _ => return None,
        };

        Some(keyword)
    }

    /// Returns whether the provided character can be considered as whitespace.
    fn is_whitespace(character: char) -> bool {
        return character == '\n' || character == '\t' || character == ' ';
    }

    /// Returns the current length offset from the source text (in UTF-8 bytes).
    fn offset(&self) -> usize {
        self.source.len() - self.chars.as_str().len()
    }

    /// Returns the next token in the source without advancing the iterator.
    fn peek(&self) -> Option<char> {
        self.chars.clone().next()
    }
}

#[cfg(test)]
mod tests {
    use petal_core::string_intern::{StringInternPoolImpl, StringReference};

    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    macro_rules! assert_tokens {
        ($string_intern_pool:expr, $source:expr $(, $e:expr)* ) => {
            assert_tokens($string_intern_pool, $source, &vec![$($e),*])
        };
    }

    #[test]
    fn test_empty_file() {
        let mut string_intern_pool = StringInternPoolImpl::new();
        assert_tokens!(&mut string_intern_pool, "");
    }

    #[test]
    fn test_identifier() {
        let mut string_intern_pool: StringInternPoolImpl = StringInternPoolImpl::new();
        let identifier_reference = StringReference(0);

        assert_tokens!(
            &mut string_intern_pool,
            "this_is_an_identifier",
            Token {
                kind: TokenKind::Identifier(identifier_reference),
                span: SourceSpan { start: 0, end: 21 }
            }
        );

        assert_eq!(
            string_intern_pool.resolve_reference(&identifier_reference),
            Some("this_is_an_identifier")
        )
    }

    #[test]
    fn test_identifier_with_numeric_characters() {
        let mut string_intern_pool: StringInternPoolImpl = StringInternPoolImpl::new();
        let identifier_reference = StringReference(0);

        assert_tokens!(
            &mut string_intern_pool,
            "i32_abc",
            Token {
                kind: TokenKind::Identifier(identifier_reference),
                span: SourceSpan { start: 0, end: 7 },
            }
        );

        assert_eq!(
            string_intern_pool.resolve_reference(&identifier_reference),
            Some("i32_abc")
        );
    }

    #[test]
    fn test_integer_literal() {
        let mut string_intern_pool = StringInternPoolImpl::new();

        assert_tokens!(
            &mut string_intern_pool,
            "512",
            Token {
                kind: TokenKind::IntegerLiteral(512),
                span: SourceSpan { start: 0, end: 3 }
            }
        );
    }

    #[test]
    fn test_zero_integer_literal() {
        let mut string_intern_pool = StringInternPoolImpl::new();

        assert_tokens!(
            &mut string_intern_pool,
            "0",
            Token {
                kind: TokenKind::IntegerLiteral(0),
                span: SourceSpan { start: 0, end: 1 }
            }
        )
    }

    #[test]
    fn test_invalid_integer_literal() {
        let mut string_intern_pool: StringInternPoolImpl = StringInternPoolImpl::new();
        let mut lexer = Lexer::new(&mut string_intern_pool, "123456789123456789123456789123456789");

        assert_eq!(
            lexer.next_token(),
            Err(Error::new(
                LexerErrorKind::InvalidIntegerLiteral,
                SourceSpan { start: 0, end: 36 }
            ))
        )
    }

    #[test]
    fn test_comment() {
        let mut string_intern_pool: StringInternPoolImpl = StringInternPoolImpl::new();
        let comment_reference = StringReference(0);

        assert_tokens!(
            &mut string_intern_pool,
            "// Hello, world!",
            Token {
                kind: TokenKind::Comment(comment_reference),
                span: SourceSpan { start: 0, end: 16 },
            }
        );

        assert_eq!(
            string_intern_pool.resolve_reference(&comment_reference),
            Some("Hello, world!")
        );
    }

    #[test]
    fn test_comment_with_weird_spacing() {
        let mut string_intern_pool: StringInternPoolImpl = StringInternPoolImpl::new();
        let comment_reference = StringReference(0);

        assert_tokens!(
            &mut string_intern_pool,
            "//    Hello, world!\n;",
            Token {
                kind: TokenKind::Comment(StringReference(0)),
                span: SourceSpan { start: 0, end: 19 },
            },
            Token {
                kind: TokenKind::Semicolon,
                span: SourceSpan { start: 20, end: 21 },
            }
        );

        assert_eq!(
            string_intern_pool.resolve_reference(&comment_reference),
            Some("    Hello, world!")
        );
    }

    #[test]
    fn test_forward_slash() {
        let mut string_intern_pool: StringInternPoolImpl = StringInternPoolImpl::new();

        assert_tokens!(
            &mut string_intern_pool,
            "/",
            Token {
                kind: TokenKind::ForwardSlash,
                span: SourceSpan { start: 0, end: 1 }
            }
        )
    }

    #[test]
    fn test_variable_declaration() {
        let mut string_intern_pool = StringInternPoolImpl::new();
        let type_reference = StringReference(0);
        let identifier_reference = StringReference(1);

        assert_tokens!(
            &mut string_intern_pool,
            "i32 identifier = 123456789;",
            Token {
                kind: TokenKind::Identifier(type_reference),
                span: SourceSpan { start: 0, end: 3 }
            },
            Token {
                kind: TokenKind::Identifier(identifier_reference),
                span: SourceSpan { start: 4, end: 14 }
            },
            Token {
                kind: TokenKind::Equals,
                span: SourceSpan { start: 15, end: 16 }
            },
            Token {
                kind: TokenKind::IntegerLiteral(123456789),
                span: { SourceSpan { start: 17, end: 26 } }
            },
            Token {
                kind: TokenKind::Semicolon,
                span: SourceSpan { start: 26, end: 27 }
            }
        );

        assert_eq!(string_intern_pool.resolve_reference(&type_reference), Some("i32"));

        assert_eq!(
            string_intern_pool.resolve_reference(&identifier_reference),
            Some("identifier")
        );
    }

    /// A helper method to assert that the tokens in the provided [Vec] can be  consumed from a [Lexer] that has been
    /// initialized with the provided `source` text.
    ///
    /// This also asserts that the final token in the stream is the EOF token.
    fn assert_tokens<S: StringInternPool>(string_intern_pool: &mut S, source: &'static str, tokens: &Vec<Token>) {
        let mut lexer = Lexer::new(string_intern_pool, source);

        // We need to keep track of the current end index for when we assert the EOF token at the end.
        let mut current_end_index: usize = 0;

        for token in tokens {
            let lexer_token = lexer.next_token().expect("next_token should not fail!");
            assert_eq!(lexer_token, *token);

            current_end_index = lexer_token.span.end;
        }

        // Now that we've looped over all of the tokens, we can assert that the final one is the EOF token.
        assert_eq!(
            lexer.next_token().expect("next_token should not fail!"),
            Token {
                kind: TokenKind::EOF,
                span: SourceSpan {
                    start: current_end_index,
                    end: current_end_index,
                }
            }
        )
    }
}
