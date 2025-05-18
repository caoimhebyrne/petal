use crate::core::{location::Location, position::Position};
use error::LexerError;
use std::{iter::Peekable, str::Chars};
use token::{Token, TokenKind};

pub mod error;
pub mod token;

const KEYWORDS: [&str; 2] = ["func", "return"];

// The lexer for the Petal programming language.
pub struct Lexer<'a> {
    characters: Peekable<Chars<'a>>,
    position: Position,
}

impl<'a> Lexer<'a> {
    pub fn from(input: &'a str) -> Lexer<'a> {
        Lexer {
            characters: input.chars().peekable(),
            position: Position::default(),
        }
    }

    pub fn parse(&mut self) -> Result<Vec<Token>, LexerError> {
        let mut tokens = vec![];

        while let Some(character) = self.characters.peek() {
            let token = match character {
                '+' => self.token(TokenKind::Plus),
                '-' => self.token(TokenKind::Minus),
                '*' => self.token(TokenKind::Asterisk),
                '=' => self.token(TokenKind::Equals),
                ';' => self.token(TokenKind::Semicolon),
                '(' => self.token(TokenKind::OpenParenthesis),
                ')' => self.token(TokenKind::CloseParenthesis),
                '{' => self.token(TokenKind::OpenBrace),
                '}' => self.token(TokenKind::CloseBrace),
                '>' => self.token(TokenKind::GreaterThan),

                '/' => {
                    // If the next token is also a `/`, we can assume that this is a comment.
                    if let Some('/') = self.characters.peek() {
                        // I'd like to use `take_while`, but it just behaves too weirdly.
                        while let Some(_) = self.characters.next_if(|it| *it != '\n') {}

                        continue;
                    }

                    self.token(TokenKind::Slash)
                }

                '\n' => {
                    self.advance_line();
                    continue;
                }

                // Any whitespace can be skipped.
                ' ' => {
                    self.advance_column();
                    continue;
                }

                _ => {
                    if character.is_alphabetic() {
                        self.parse_identifier()?
                    } else if character.is_numeric() {
                        self.parse_integer_literal()?
                    } else {
                        let location = Location::new(self.position, 1);
                        return Err(LexerError::unexpected_character(*character, location));
                    }
                }
            };

            tokens.push(token);
        }

        Ok(tokens)
    }

    fn advance_column(&mut self) {
        self.position.column += 1;
        self.characters.next();
    }

    fn advance_line(&mut self) {
        self.position.next_line();
        self.characters.next();
    }

    fn token(&mut self, kind: TokenKind) -> Token {
        let token = Token::new(kind, Location::new(self.position, 1));

        self.advance_column();

        token
    }

    /// Attempts to parse an integer literal from the character stream.
    fn parse_integer_literal(&mut self) -> Result<Token, LexerError> {
        let starting_position = self.position;
        let mut chars = vec![];

        while let Some(character) = self.characters.next_if(|it| it.is_numeric()) {
            self.position.column += 1;
            chars.push(character);
        }

        let location = Location::new(starting_position, chars.len());
        let string_value = chars.iter().collect::<String>();

        u64::from_str_radix(&string_value, 10)
            .map_err(|_| LexerError::invalid_integer_literal(string_value, location))
            .map(|value| Token::new(TokenKind::IntegerLiteral(value), location))
    }

    /// Attempts to parse an identifier from the character stream.
    fn parse_identifier(&mut self) -> Result<Token, LexerError> {
        let starting_position = self.position;
        let mut chars = vec![];

        while let Some(character) = self
            .characters
            .next_if(|it| it.is_alphanumeric() || *it == '_')
        {
            self.position.column += 1;
            chars.push(character);
        }

        let identifier = chars.iter().collect::<String>();
        let kind = if KEYWORDS.iter().any(|it| *it == identifier) {
            TokenKind::Keyword(identifier)
        } else {
            TokenKind::Identifier(identifier)
        };

        Ok(Token::new(
            kind,
            Location::new(starting_position, chars.len()),
        ))
    }
}
