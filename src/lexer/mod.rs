use std::str::Chars;

use crate::{
    core::{error::Error, source_span::SourceSpan},
    lexer::{
        error::LexerErrorKind,
        token::{Keyword, Token, TokenKind},
    },
};

pub mod error;
pub mod token;

/// The lexer is responsible for taking an input string and producing tokens from that input.
pub struct Lexer<'a> {
    /// The source being parsed.
    source: &'a str,

    /// The remaining characters to be consumed from the source.
    chars: Chars<'a>,
}

impl<'a> Lexer<'a> {
    /// Creates a new Lexer instance.
    pub fn new(source: &'a str) -> Lexer<'a> {
        return Lexer {
            source,
            chars: source.chars(),
        };
    }

    /// Returns the token at the lexer's current position.
    pub fn next_token(&mut self) -> Result<Token, Error> {
        let start_offset = self.offset();
        let token_kind_result = self.next_kind();
        let end_offset = self.offset();

        let span = SourceSpan {
            start: start_offset,
            end: end_offset,
        };

        match token_kind_result {
            Ok(kind) => Ok(Token { kind, span }),
            Err(kind) => Err(Error {
                kind: kind.into(),
                span,
            }),
        }
    }

    /// Returns the next token kind at the lexer's current position.
    fn next_kind(&mut self) -> Result<TokenKind, LexerErrorKind> {
        while let Some(character) = self.chars.next() {
            let kind = match character {
                ' ' | '\n' | '\t' => continue,

                '=' => TokenKind::Equals,
                ';' => TokenKind::Semicolon,
                '(' => TokenKind::LeftParenthesis,
                ')' => TokenKind::RightParenthesis,
                '{' => TokenKind::LeftBrace,
                '}' => TokenKind::RightBrace,
                '-' => TokenKind::Hyphen,
                '>' => TokenKind::RightAngleBracket,

                '/' => return self.parse_forward_slash_or_comment(),

                '0'..'9' => return self.parse_integer_literal(character),

                _ => {
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
    fn parse_integer_literal(&mut self, first_character: char) -> Result<TokenKind, LexerErrorKind> {
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
    fn parse_identifier_or_keyword(&mut self, first_character: char) -> Result<TokenKind, LexerErrorKind> {
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
        let kind = if let Some(keyword) = Lexer::match_keyword(&identifier) {
            TokenKind::Keyword(keyword)
        } else {
            TokenKind::Identifier(identifier)
        };

        Ok(kind)
    }

    /// Attempts to parse a forward slash token or a commenet token from the current position in the source text.
    fn parse_forward_slash_or_comment(&mut self) -> Result<TokenKind, LexerErrorKind> {
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
        Ok(TokenKind::Comment(comment))
    }

    /// Attempts to match a keyword from the input string, returning [Option::None] if a matching keyword was not found.
    fn match_keyword(string: &str) -> Option<Keyword> {
        let keyword = match string {
            "let" => Keyword::Let,
            "func" => Keyword::Func,
            _ => return None,
        };

        Some(keyword)
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
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    macro_rules! assert_tokens {
        ($source:expr $(, $e:expr)* ) => {
            assert_tokens($source, &vec![$($e),*])
        };
    }

    #[test]
    fn test_empty_file() {
        assert_tokens!("");
    }

    #[test]
    fn test_identifier() {
        assert_tokens!(
            "this_is_an_identifier",
            Token {
                kind: TokenKind::Identifier("this_is_an_identifier".to_string()),
                span: SourceSpan { start: 0, end: 21 }
            }
        );
    }

    #[test]
    fn test_identifier_with_numeric_characters() {
        assert_tokens!(
            "i32_abc",
            Token {
                kind: TokenKind::Identifier("i32_abc".to_string()),
                span: SourceSpan { start: 0, end: 7 },
            }
        )
    }

    #[test]
    fn test_integer_literal() {
        assert_tokens!(
            "512",
            Token {
                kind: TokenKind::IntegerLiteral(512),
                span: SourceSpan { start: 0, end: 3 }
            }
        );
    }

    #[test]
    fn test_zero_integer_literal() {
        assert_tokens!(
            "0",
            Token {
                kind: TokenKind::IntegerLiteral(0),
                span: SourceSpan { start: 0, end: 1 }
            }
        )
    }

    #[test]
    fn test_invalid_integer_literal() {
        let mut lexer = Lexer::new("123456789123456789123456789123456789");

        assert_eq!(
            lexer.next_token(),
            Err(Error {
                kind: LexerErrorKind::InvalidIntegerLiteral.into(),
                span: SourceSpan { start: 0, end: 36 },
            })
        )
    }

    #[test]
    fn test_comment() {
        assert_tokens!(
            "// Hello, world!",
            Token {
                kind: TokenKind::Comment("Hello, world!".to_string()),
                span: SourceSpan { start: 0, end: 16 },
            }
        )
    }

    #[test]
    fn test_comment_with_weird_spacing() {
        assert_tokens!(
            "//    Hello, world!\n;",
            Token {
                kind: TokenKind::Comment("    Hello, world!".to_string()),
                span: SourceSpan { start: 0, end: 19 },
            },
            Token {
                kind: TokenKind::Semicolon,
                span: SourceSpan { start: 19, end: 21 },
            }
        )
    }

    #[test]
    fn test_forward_slash() {
        assert_tokens!(
            "/",
            Token {
                kind: TokenKind::ForwardSlash,
                span: SourceSpan { start: 0, end: 1 }
            }
        )
    }

    #[test]
    fn test_variable_assignment() {
        assert_tokens!(
            "let identifier = 123456789;",
            Token {
                kind: TokenKind::Keyword(Keyword::Let),
                span: SourceSpan { start: 0, end: 3 }
            },
            Token {
                kind: TokenKind::Identifier("identifier".to_string()),
                span: SourceSpan { start: 3, end: 14 }
            },
            Token {
                kind: TokenKind::Equals,
                span: SourceSpan { start: 14, end: 16 }
            },
            Token {
                kind: TokenKind::IntegerLiteral(123456789),
                span: { SourceSpan { start: 16, end: 26 } }
            },
            Token {
                kind: TokenKind::Semicolon,
                span: SourceSpan { start: 26, end: 27 }
            }
        );
    }

    /// A helper method to assert that the tokens in the provided [Vec] can be  consumed from a [Lexer] that has been
    /// initialized with the provided `source` text.
    ///
    /// This also asserts that the final token in the stream is the EOF token.
    fn assert_tokens(source: &'static str, tokens: &Vec<Token>) {
        let mut lexer = Lexer::new(source);

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
