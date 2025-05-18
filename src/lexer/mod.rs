use error::LexerError;
use std::{iter::Peekable, str::Chars};
use token::{Token, TokenKind};

pub mod error;
pub mod token;

const KEYWORDS: [&str; 1] = ["func"];

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
                '=' => Token::new(TokenKind::Equals),
                ';' => Token::new(TokenKind::Semicolon),
                '(' => Token::new(TokenKind::OpenParenthesis),
                ')' => Token::new(TokenKind::CloseParenthesis),
                '{' => Token::new(TokenKind::OpenBrace),
                '}' => Token::new(TokenKind::CloseBrace),
                '>' => Token::new(TokenKind::GreaterThan),

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
                    if character.is_alphabetic() {
                        self.parse_identifier(character)?
                    } else if character.is_numeric() {
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

    /// Attempts to parse an identifier from the character stream.
    fn parse_identifier(&mut self, initial_character: char) -> Result<Token, LexerError> {
        let mut chars = vec![initial_character];

        while let Some(character) = self
            .characters
            .next_if(|it| it.is_alphanumeric() || *it == '_')
        {
            chars.push(character);
        }

        let identifier = chars.iter().collect::<String>();
        let kind = if KEYWORDS.iter().any(|it| *it == identifier) {
            TokenKind::Keyword(identifier)
        } else {
            TokenKind::Identifier(identifier)
        };

        Ok(Token::new(kind))
    }
}
