use std::str::Chars;

use crate::{
    core::span::Span,
    lexer::{
        error::{
            LexerError,
            LexerErrorKind,
        },
        token::{
            Keyword,
            Token,
            TokenKind,
        },
    },
    module_registry::ModuleId,
};

pub mod error;
pub mod token;

/// A basic [`Lexer`] for the Petal programming language.
pub struct Lexer<'a> {
    /// The ID of the module being parsed.
    module_id: ModuleId,

    /// The contents of the source file to parse.
    source: Chars<'a>,

    /// The index that the lexer is currently at within the source file.
    cursor: usize,
}

impl<'a> Lexer<'a> {
    /// Creates a new [`Lexer`].
    pub fn new(module_id: ModuleId, source: &'a str) -> Self {
        Lexer { module_id, source: source.chars(), cursor: 0 }
    }

    /// Attempts to parse the source code within this [`Lexer`] into a [`Vec`] of [`Token`]s.
    pub fn parse(&mut self) -> Result<Vec<Token>, LexerError> {
        let mut tokens = vec![];

        while let Some(character) = self.next() {
            let token = match character {
                '(' => Token::new(TokenKind::OpenParen, self.span(1)),
                ')' => Token::new(TokenKind::CloseParen, self.span(1)),
                '-' => Token::new(TokenKind::Hyphen, self.span(1)),
                '<' => Token::new(TokenKind::LeftAngleBracket, self.span(1)),
                '>' => Token::new(TokenKind::RightAngleBracket, self.span(1)),
                '{' => Token::new(TokenKind::OpenBrace, self.span(1)),
                '}' => Token::new(TokenKind::CloseBrace, self.span(1)),
                ';' => Token::new(TokenKind::Semicolon, self.span(1)),
                ',' => Token::new(TokenKind::Comma, self.span(1)),
                '+' => Token::new(TokenKind::Plus, self.span(1)),
                '*' => Token::new(TokenKind::Asterisk, self.span(1)),
                '~' => Token::new(TokenKind::Tilda, self.span(1)),
                '&' => Token::new(TokenKind::Ampersand, self.span(1)),
                '@' => Token::new(TokenKind::At, self.span(1)),
                '.' => Token::new(TokenKind::Period, self.span(1)),
                '?' => Token::new(TokenKind::QuestionMark, self.span(1)),

                '!' => {
                    if let Some('=') = self.peek() {
                        self.next();
                        Token::new(TokenKind::NotEquals, self.span(2))
                    } else {
                        Token::new(TokenKind::ExclamationMark, self.span(1))
                    }
                }

                '=' => {
                    if let Some('=') = self.peek() {
                        self.next();
                        Token::new(TokenKind::DoubleEquals, self.span(2))
                    } else {
                        Token::new(TokenKind::Equals, self.span(1))
                    }
                }

                ':' => {
                    if let Some(':') = self.peek() {
                        self.next();
                        Token::new(TokenKind::DoubleColon, self.span(2))
                    } else {
                        Token::new(TokenKind::Colon, self.span(1))
                    }
                }

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

                '"' => self.parse_string_literal()?,

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

    /// Attempts to parse a string literal token at the [`Lexer`]'s current position.
    ///
    /// The opening quote is expected to have already been consumed by the caller. Escape sequences
    /// are decoded as they are read, so the resulting token holds the string's actual value rather
    /// than the characters used to spell it out in the source file.
    fn parse_string_literal(&mut self) -> Result<Token, LexerError> {
        // The caller has already consumed the opening quote, so it sits just behind the cursor.
        let start = self.cursor - 1;

        let mut string = String::new();

        loop {
            // A string literal may not span multiple lines, so a new-line is treated the same way
            // as the end of the source file: the literal was never terminated.
            let Some(character) = self.peek().filter(|it| *it != '\n') else {
                return Err(LexerErrorKind::UnterminatedStringLiteral.at(self.span_from(start)));
            };

            self.next();

            match character {
                '"' => break,
                '\\' => string.push(self.parse_escape_sequence()?),
                _ => string.push(character),
            }
        }

        Ok(Token::new(TokenKind::String(string), self.span_from(start)))
    }

    /// Attempts to parse the remainder of an escape sequence within a string literal, returning the
    /// character that the sequence represents.
    ///
    /// The leading backslash is expected to have already been consumed by the caller.
    fn parse_escape_sequence(&mut self) -> Result<char, LexerError> {
        // The backslash sits just behind the cursor, and is part of the sequence being parsed.
        let start = self.cursor - 1;

        let Some(character) = self.next().filter(|it| *it != '\n') else {
            return Err(LexerErrorKind::UnterminatedStringLiteral.at(self.span_from(start)));
        };

        match character {
            '\\' => Ok('\\'),
            '"' => Ok('"'),
            'n' => Ok('\n'),
            'r' => Ok('\r'),
            't' => Ok('\t'),
            '0' => Ok('\0'),

            _ => Err(LexerErrorKind::InvalidEscapeSequence(character).at(self.span_from(start))),
        }
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
        Span::new(self.module_id, self.cursor - length, length)
    }

    /// Returns a [`Span`] covering the source between the provided start index and the [`Lexer`]'s
    /// current cursor.
    fn span_from(&self, start: usize) -> Span {
        Span::new(self.module_id, start, self.cursor - start)
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::{
        lexer::token::Keyword,
        module_registry::MOCK_MODULE_ID,
    };

    fn assert_lexer_tokens(source: &str, tokens: Vec<Token>) {
        let mut lexer = Lexer::new(MOCK_MODULE_ID, source);
        assert_eq!(lexer.parse(), Ok(tokens));
    }

    #[test]
    fn parse_opening_and_closing_parenthesis_with_whitespace() {
        assert_lexer_tokens(
            "(   )",
            vec![
                Token::new(TokenKind::OpenParen, Span::new(MOCK_MODULE_ID, 0, 1)),
                Token::new(TokenKind::CloseParen, Span::new(MOCK_MODULE_ID, 4, 1)),
            ],
        )
    }

    #[test]
    fn parse_integer_number_literal() {
        assert_lexer_tokens("123456", vec![Token::new(TokenKind::Number(123456.0), Span::new(MOCK_MODULE_ID, 0, 6))])
    }

    #[test]
    fn parse_float_number_literal() {
        assert_lexer_tokens("123.456", vec![Token::new(TokenKind::Number(123.456), Span::new(MOCK_MODULE_ID, 0, 7))]);
    }

    #[test]
    fn parse_identifier() {
        assert_lexer_tokens(
            "identifier",
            vec![Token::new(TokenKind::Identifier("identifier".into()), Span::new(MOCK_MODULE_ID, 0, 10))],
        );
    }

    #[test]
    fn parse_func_keyword() {
        assert_lexer_tokens(
            "func",
            vec![Token::new(TokenKind::Keyword(Keyword::Func), Span::new(MOCK_MODULE_ID, 0, 4))],
        );
    }

    #[test]
    fn parse_variable_declaration() {
        assert_lexer_tokens(
            "i32 identifier = 100",
            vec![
                Token::new(TokenKind::Identifier("i32".into()), Span::new(MOCK_MODULE_ID, 0, 3)),
                Token::new(TokenKind::Identifier("identifier".into()), Span::new(MOCK_MODULE_ID, 4, 10)),
                Token::new(TokenKind::Equals, Span::new(MOCK_MODULE_ID, 15, 1)),
                Token::new(TokenKind::Number(100.0), Span::new(MOCK_MODULE_ID, 17, 3)),
            ],
        );
    }

    #[test]
    fn parse_basic_function_declaration() {
        assert_lexer_tokens(
            "func test() {}",
            vec![
                Token::new(TokenKind::Keyword(Keyword::Func), Span::new(MOCK_MODULE_ID, 0, 4)),
                Token::new(TokenKind::Identifier("test".into()), Span::new(MOCK_MODULE_ID, 5, 4)),
                Token::new(TokenKind::OpenParen, Span::new(MOCK_MODULE_ID, 9, 1)),
                Token::new(TokenKind::CloseParen, Span::new(MOCK_MODULE_ID, 10, 1)),
                Token::new(TokenKind::OpenBrace, Span::new(MOCK_MODULE_ID, 12, 1)),
                Token::new(TokenKind::CloseBrace, Span::new(MOCK_MODULE_ID, 13, 1)),
            ],
        );
    }

    #[test]
    fn parse_return() {
        assert_lexer_tokens(
            "return 123;",
            vec![
                Token::new(TokenKind::Keyword(Keyword::Return), Span::new(MOCK_MODULE_ID, 0, 6)),
                Token::new(TokenKind::Number(123.0), Span::new(MOCK_MODULE_ID, 7, 3)),
                Token::new(TokenKind::Semicolon, Span::new(MOCK_MODULE_ID, 10, 1)),
            ],
        );
    }

    #[test]
    fn skips_comments_but_retains_forward_slash() {
        assert_lexer_tokens(
            "// This is a test!\n//This is another test!\n/",
            vec![Token::new(TokenKind::ForwardSlash, Span::new(MOCK_MODULE_ID, 43, 1))],
        );
    }

    #[test]
    fn parse_string_literal() {
        assert_lexer_tokens(
            r#""hello""#,
            vec![Token::new(TokenKind::String("hello".into()), Span::new(MOCK_MODULE_ID, 0, 7))],
        );
    }

    #[test]
    fn parse_empty_string_literal() {
        assert_lexer_tokens(r#""""#, vec![Token::new(TokenKind::String("".into()), Span::new(MOCK_MODULE_ID, 0, 2))]);
    }

    #[test]
    fn parse_string_literal_escape_sequences() {
        // The span covers the source text of the literal (14 characters), whereas the value that it
        // decodes to is only 8 characters long.
        assert_lexer_tokens(
            r#""a\n\t\r\0\\\"""#,
            vec![Token::new(TokenKind::String("a\n\t\r\0\\\"".into()), Span::new(MOCK_MODULE_ID, 0, 15))],
        );
    }

    #[test]
    fn error_invalid_escape_sequence() {
        let mut lexer = Lexer::new(MOCK_MODULE_ID, r#""oh \q no""#);
        assert_eq!(
            lexer.parse(),
            Err(LexerError::new(LexerErrorKind::InvalidEscapeSequence('q'), Span::new(MOCK_MODULE_ID, 4, 2)))
        );
    }

    #[test]
    fn error_unterminated_string_literal_at_end_of_source() {
        let mut lexer = Lexer::new(MOCK_MODULE_ID, r#""unterminated"#);
        assert_eq!(
            lexer.parse(),
            Err(LexerError::new(LexerErrorKind::UnterminatedStringLiteral, Span::new(MOCK_MODULE_ID, 0, 13)))
        );
    }

    #[test]
    fn error_unterminated_string_literal_at_new_line() {
        let mut lexer = Lexer::new(MOCK_MODULE_ID, "\"unterminated\n\"");
        assert_eq!(
            lexer.parse(),
            Err(LexerError::new(LexerErrorKind::UnterminatedStringLiteral, Span::new(MOCK_MODULE_ID, 0, 13)))
        );
    }

    #[test]
    fn error_unexpected_character() {
        let mut lexer = Lexer::new(MOCK_MODULE_ID, "\u{200b}");
        assert_eq!(
            lexer.parse(),
            Err(LexerError::new(LexerErrorKind::UnrecognizedCharacter('\u{200b}'), Span::new(MOCK_MODULE_ID, 0, 1)))
        );
    }
}
