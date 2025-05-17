use std::{
    iter::{self, Peekable},
    str::Chars,
};

use error::LexerError;
use token::{Token, TokenKind};

pub mod error;
pub mod token;

// The lexer for the Petal programming language.
pub struct Lexer<'a> {
    characters: Peekable<Chars<'a>>,
}

impl<'a> Lexer<'a> {
    pub fn from(input: &'a str) -> Lexer<'a> {
        Lexer {
            characters: input.chars().peekable(),
        }
    }

    pub fn parse(&mut self) -> Result<Vec<Token>, LexerError> {
        let mut tokens = vec![];

        while let Some(character) = self.consume() {
            let token = match character {
                '+' => Token::new(TokenKind::Plus),
                '-' => Token::new(TokenKind::Minus),
                '*' => Token::new(TokenKind::Asterisk),

                '/' => {
                    // If the next token is also a `/`, we can assume that this is a comment.
                    if let Some('/') = self.characters.peek() {
                        // I'd like to use `take_while`, but it just behaves too weirdly.
                        while let Some(_) = self.characters.next_if(|it| *it != '\n') {}

                        continue;
                    }

                    Token::new(TokenKind::Slash)
                }

                // Any whitespace can be skipped.
                '\n' | ' ' => continue,

                _ => {
                    if character.is_numeric() {
                        self.parse_integer_literal(character)?
                    } else {
                        return Err(LexerError::unexpected_character(character));
                    }
                }
            };

            tokens.push(token);
        }

        Ok(tokens)
    }

    /// Attempts to consume a character from the stream, returning None if the
    /// end of the stream has been reached.
    fn consume(&mut self) -> Option<char> {
        self.characters.next()
    }

    /// Attempts to parse an integer literal from the character stream.
    fn parse_integer_literal(&mut self, initial_character: char) -> Result<Token, LexerError> {
        let mut chars = vec![initial_character];

        while let Some(character) = self.characters.next_if(|it| it.is_numeric()) {
            chars.push(character);
        }

        let string_value = chars.iter().collect::<String>();

        u64::from_str_radix(&string_value, 10)
            .map_err(|_| LexerError::invalid_integer_literal(string_value))
            .map(|value| Token::new(TokenKind::IntegerLiteral(value)))
    }
}
