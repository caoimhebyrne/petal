use crate::core::{location::Location, position::Position};
use error::LexerError;
use std::{collections::HashMap, str::Chars};
use token::{Keyword, Token, TokenKind};

pub mod error;
pub mod token;

type LexerResult<T> = Result<T, LexerError>;

pub struct Lexer<'a> {
    // The remaining characters.
    input: Chars<'a>,

    // The position that the lexer is at within the input stream.
    position: Position,

    // The reserved identifiers to be treated as keywords.
    keywords: HashMap<String, Keyword>,
}

impl<'a> Lexer<'a> {
    // Initializes a new Lexer with the provided string as the input.
    pub fn new(source: &'a str) -> Self {
        Self {
            input: source.chars(),
            position: Position::default(),
            keywords: HashMap::from([
                ("func".to_owned(), Keyword::Func),
                ("return".to_owned(), Keyword::Return),
                ("extern".to_owned(), Keyword::Extern),
                ("if".to_owned(), Keyword::If),
                ("else".to_owned(), Keyword::Else),
                ("true".to_owned(), Keyword::True),
                ("false".to_owned(), Keyword::False),
            ]),
        }
    }

    // Attempts to parse the input of this Lexer into a vec of tokens.
    pub fn parse(&mut self) -> LexerResult<Vec<Token>> {
        let mut tokens = vec![];

        while let Some((character, position)) = self.next() {
            let token = match character {
                '+' => Token::new(TokenKind::Plus, position.into()),
                '-' => Token::new(TokenKind::Minus, position.into()),
                '*' => Token::new(TokenKind::Asterisk, position.into()),
                '=' => Token::new(TokenKind::Equals, position.into()),
                ':' => Token::new(TokenKind::Colon, position.into()),
                ';' => Token::new(TokenKind::Semicolon, position.into()),
                '(' => Token::new(TokenKind::OpenParenthesis, position.into()),
                ')' => Token::new(TokenKind::CloseParenthesis, position.into()),
                '{' => Token::new(TokenKind::OpenBrace, position.into()),
                '}' => Token::new(TokenKind::CloseBrace, position.into()),
                '<' => Token::new(TokenKind::LessThan, position.into()),
                '>' => Token::new(TokenKind::GreaterThan, position.into()),
                ',' => Token::new(TokenKind::Comma, position.into()),
                '&' => Token::new(TokenKind::Ampersand, position.into()),

                '"' => self.parse_string_literal(position)?,

                '/' => {
                    if let Some('/') = self.peek() {
                        // This is a comment, we should consume all characters until a new-line is reached.
                        while let Some((character, _)) = self.next() {
                            if character == '\n' {
                                break;
                            }
                        }

                        continue;
                    } else {
                        Token::new(TokenKind::Slash, position.into())
                    }
                }

                '\n' | ' ' => continue,

                _ => {
                    if character.is_alphabetic() {
                        self.parse_identifier_token(character, position)?
                    } else if character.is_numeric() {
                        self.parse_integer_literal_token(character, position)?
                    } else {
                        return Err(LexerError::unexpected_character(character, position.into()));
                    }
                }
            };

            tokens.push(token);
        }

        Ok(tokens)
    }

    // Attempts to read the next character in the input stream while advancing the iterator.
    fn next(&mut self) -> Option<(char, Position)> {
        self.input.next().map(|character| {
            let position = self.position;

            if character == '\n' {
                self.position.line += 1;
                self.position.column = 0;
            } else {
                self.position.column += 1;
            }

            (character, position)
        })
    }

    // Attempts to read the next character in the input stream without advancing the iterator.
    // Using Peekable is slow, and this `clone` is cheap as it only copies the tracking and boundary index.
    fn peek(&self) -> Option<char> {
        self.input.clone().next()
    }

    // Attempts to parse an identifier at the lexer's current position.
    fn parse_identifier_token(&mut self, start_character: char, start_position: Position) -> LexerResult<Token> {
        let identifier = self.read_string(Some(start_character), |it| it.is_alphanumeric() || it == '_');

        // If this is a reserved identifier, we must emit it as a keyword.
        let kind = match self.keywords.get(&identifier) {
            Some(keyword) => TokenKind::Keyword(*keyword),
            None => TokenKind::Identifier(identifier),
        };

        Ok(Token::new(kind, Location::between(start_position, self.position)))
    }

    // Attempts to parse an integer literal at the lexer's current position.
    fn parse_integer_literal_token(&mut self, start_character: char, start_position: Position) -> LexerResult<Token> {
        let literal = self.read_string(Some(start_character), |it| it.is_numeric());

        let location = Location::between(start_position, self.position);
        let integer = literal
            .parse::<u64>()
            .map_err(|_| LexerError::invalid_integer_literal(literal, location))?;

        Ok(Token::new(TokenKind::IntegerLiteral(integer), location))
    }

    // Attempts to parse a string literal at the lexer's current position.
    fn parse_string_literal(&mut self, start_position: Position) -> LexerResult<Token> {
        let value = self.read_string(None, |it| it != '"');

        if let Some((character, end_position)) = self.next() {
            if character != '"' {
                return Err(LexerError::unexpected_character(character, end_position.into()));
            }

            return Ok(Token::new(
                TokenKind::StringLiteral(value),
                Location::between(start_position, end_position),
            ));
        }

        Err(LexerError::unexpected_end_of_file(self.position.into()))
    }

    // Produces a string by reading characters from the token stream while the provided predicate is true,
    // or the lexer runs out of characters.
    fn read_string<F>(&mut self, start_character: Option<char>, predicate: F) -> String
    where
        F: Fn(char) -> bool,
    {
        let mut characters = vec![];
        if let Some(char) = start_character {
            characters.push(char);
        }

        while let Some(character) = self.peek() {
            if !predicate(character) {
                break;
            }

            self.next();

            if character == '\\' {
                if let Some('n') = self.peek() {
                    self.next();
                    characters.push('\n');

                    continue;
                }
            }

            characters.push(character);
        }

        characters.iter().collect::<String>()
    }
}
