use std::str::Chars;

use crate::{
    core::span::Span,
    lexer::{
        error::{LexerError, LexerErrorKind},
        token::{Keyword, Token, TokenKind},
    },
};

pub mod error;
pub mod token;

/// A basic [`Lexer`] for the Petal programming language.
pub struct Lexer<'a> {
    /// The contents of the source file to parse.
    source: Chars<'a>,

    /// The index that the lexer is currently at within the source file.
    cursor: usize,
}

impl<'a> Lexer<'a> {
    /// Creates a new [`Lexer`].
    pub fn new(source: &'a str) -> Self {
        Lexer { source: source.chars(), cursor: 0 }
    }

    /// Attempts to parse the source code within this [`Lexer`] into a [`Vec`] of [`Token`]s.
    pub fn parse(&mut self) -> Result<Vec<Token>, LexerError> {
        let mut tokens = vec![];

        while let Some(character) = self.next() {
            let token = match character {
                '(' => Token::new(TokenKind::OpenParen, self.span(1)),
                ')' => Token::new(TokenKind::CloseParen, self.span(1)),
                '=' => Token::new(TokenKind::Equals, self.span(1)),
                '-' => Token::new(TokenKind::Hyphen, self.span(1)),
                '>' => Token::new(TokenKind::RightAngleBracket, self.span(1)),
                '{' => Token::new(TokenKind::OpenBrace, self.span(1)),
                '}' => Token::new(TokenKind::CloseBrace, self.span(1)),
                ';' => Token::new(TokenKind::Semicolon, self.span(1)),

                '/' => {
                    // If the next token is also a slash, then this is a comment. We should keep
                    // reading it until a new-line occurs.
                    if let Some('/') = self.peek() {
                        self.next();

                        let mut comment = String::new();
                        self.consume_while(&mut comment, |it| it != '\n');

                        continue;
                    } else {
                        Token::new(TokenKind::ForwardSlash, self.span(1))
                    }
                }

                ' ' | '\n' => continue,

                _ => {
                    if character.is_numeric() {
                        self.parse_number_literal(character)?
                    } else if character.is_alphabetic() {
                        self.parse_identifier_or_keyword(character)?
                    } else {
                        return Err(LexerError::new(LexerErrorKind::UnrecognizedCharacter(character), self.span(1)));
                    }
                }
            };

            tokens.push(token);
        }

        Ok(tokens)
    }

    /// Attempts to parse a number literal token at the [`Lexer`]'s current position, taking the
    /// provided character as the start of the number literal.
    fn parse_number_literal(&mut self, start: char) -> Result<Token, LexerError> {
        let mut number_string = format!("{}", start);

        // We can consume characters as long as they are compatible with our number literal format.
        self.consume_while(&mut number_string, |char| char.is_numeric() || char == '.');

        let token_span = self.span(number_string.len());

        // Then, we can attempt to parse a number from the final result.
        let value = number_string
            .parse::<f64>()
            .map_err(|_| LexerError::new(LexerErrorKind::InvalidNumberLiteral(number_string), token_span))?;

        Ok(Token::new(TokenKind::Number(value), token_span))
    }

    /// Attempts to parse an identifier at the start of the [`Lexer`]'s current position, taking
    /// the provided character as the start of the identifier.
    ///
    /// If the parsed identifier is a keyword, then a keyword token will be produced instead of a
    /// regular identifier token.
    fn parse_identifier_or_keyword(&mut self, start: char) -> Result<Token, LexerError> {
        let mut identifier_string = format!("{}", start);

        // We can consume characters as long as they are compatible with our identifier's format.
        self.consume_while(&mut identifier_string, |char| char.is_alphanumeric() || char == '_');

        let token_span = self.span(identifier_string.len());

        // If the identifier is a reserved keyword, then we can return that token kind instead.
        let kind = match Keyword::from(&identifier_string) {
            Some(keyword) => TokenKind::Keyword(keyword),
            _ => TokenKind::Identifier(identifier_string),
        };

        Ok(Token::new(kind, token_span))
    }

    /// Attempts to return the character at this [`Lexer`]'s current position whle advancing the
    /// iterator. Returns [`None`] if the end of the source was reached.
    fn next(&mut self) -> Option<char> {
        self.source.next().inspect(|_| self.cursor += 1)
    }

    /// Attempts to return the character at this [`Lexer`]'s current position without advancing the
    /// iterator. Returns [`None`] if the end of the source was reached.
    fn peek(&mut self) -> Option<char> {
        // This clone is relatively cheap: https://oxc-project.github.io/javascript-parser-in-rust/docs/lexer#peek.
        self.source.clone().next()
    }

    /// Attempts to collect all characters matching the provided predicate into the provided
    /// [`String`]. This function will stop appending to the string once a character is reached
    /// that does not match the predicate.
    fn consume_while<P>(&mut self, output: &mut String, predicate: P)
    where
        P: Fn(char) -> bool,
    {
        while let Some(character) = self.peek() {
            if !predicate(character) {
                break;
            }

            output.push(self.next().expect("next should succeed if peek does?"))
        }
    }

    /// Returns a [`Span`] for the [`Lexer`]'s current cursor.
    fn span(&self, length: usize) -> Span {
        // Subtracting the length is typically safe here. The cursor is always advanced before
        // `self.span` is called (see `self.next`), so we should never be performing a subtraction
        // that ends in a negative number.
        //
        // The only exception to that is a bad input, but bugs for that should be caught by the
        // unit tests.
        Span { start: self.cursor - length, length }
    }
}

#[cfg(test)]
mod tests {
    use crate::lexer::token::Keyword;

    use super::*;
    use pretty_assertions::assert_eq;

    fn assert_lexer_tokens(source: &str, tokens: Vec<Token>) {
        let mut lexer = Lexer::new(source.into());
        assert_eq!(lexer.parse(), Ok(tokens));
    }

    #[test]
    fn parse_opening_and_closing_parenthesis_with_whitespace() {
        assert_lexer_tokens(
            "(   )",
            vec![
                Token::new(TokenKind::OpenParen, Span { start: 0, length: 1 }),
                Token::new(TokenKind::CloseParen, Span { start: 4, length: 1 }),
            ],
        )
    }

    #[test]
    fn parse_integer_number_literal() {
        assert_lexer_tokens("123456", vec![Token::new(TokenKind::Number(123456.0), Span { start: 0, length: 6 })])
    }

    #[test]
    fn parse_float_number_literal() {
        assert_lexer_tokens("123.456", vec![Token::new(TokenKind::Number(123.456), Span { start: 0, length: 7 })]);
    }

    #[test]
    fn parse_identifier() {
        assert_lexer_tokens(
            "identifier",
            vec![Token::new(TokenKind::Identifier("identifier".into()), Span { start: 0, length: 10 })],
        );
    }

    #[test]
    fn parse_func_keyword() {
        assert_lexer_tokens("func", vec![Token::new(TokenKind::Keyword(Keyword::Func), Span { start: 0, length: 4 })]);
    }

    #[test]
    fn parse_variable_declaration() {
        assert_lexer_tokens(
            "i32 identifier = 100",
            vec![
                Token::new(TokenKind::Identifier("i32".into()), Span { start: 0, length: 3 }),
                Token::new(TokenKind::Identifier("identifier".into()), Span { start: 4, length: 10 }),
                Token::new(TokenKind::Equals, Span { start: 15, length: 1 }),
                Token::new(TokenKind::Number(100.0), Span { start: 17, length: 3 }),
            ],
        );
    }

    #[test]
    fn parse_basic_function_declaration() {
        assert_lexer_tokens(
            "func test() {}",
            vec![
                Token::new(TokenKind::Keyword(Keyword::Func), Span { start: 0, length: 4 }),
                Token::new(TokenKind::Identifier("test".into()), Span { start: 5, length: 4 }),
                Token::new(TokenKind::OpenParen, Span { start: 9, length: 1 }),
                Token::new(TokenKind::CloseParen, Span { start: 10, length: 1 }),
                Token::new(TokenKind::OpenBrace, Span { start: 12, length: 1 }),
                Token::new(TokenKind::CloseBrace, Span { start: 13, length: 1 }),
            ],
        );
    }

    #[test]
    fn parse_return() {
        assert_lexer_tokens(
            "return 123;",
            vec![
                Token::new(TokenKind::Keyword(Keyword::Return), Span { start: 0, length: 6 }),
                Token::new(TokenKind::Number(123.0), Span { start: 7, length: 3 }),
                Token::new(TokenKind::Semicolon, Span { start: 10, length: 1 }),
            ],
        );
    }

    #[test]
    fn skips_comments_but_retains_forward_slash() {
        assert_lexer_tokens(
            "// This is a test!\n//This is another test!\n/",
            vec![Token::new(TokenKind::ForwardSlash, Span { start: 43, length: 1 })],
        );
    }

    #[test]
    fn error_unexpected_character() {
        let mut lexer = Lexer::new("\u{200b}".into());
        assert_eq!(
            lexer.parse(),
            Err(LexerError::new(LexerErrorKind::UnrecognizedCharacter('\u{200b}'), Span { start: 0, length: 1 }))
        );
    }
}
