use crate::{
    ast::{
        error::{ASTError, ASTErrorKind},
        statement::{Statement, function_declaration::FunctionDeclaration},
    },
    core::span::Span,
    lexer::token::{Keyword, Token, TokenKind},
};

pub mod error;
pub mod statement;

/// The AST parser.
pub struct ASTParser {
    /// The tokens to parse into an AST.
    tokens: Vec<Token>,

    /// The position that the parser is at within the [tokens].
    cursor: usize,
}

impl ASTParser {
    /// Creates a new [ASTParser].
    pub fn new(tokens: Vec<Token>) -> Self {
        ASTParser { tokens, cursor: 0 }
    }

    /// Creates a new [ASTParser] instance and parses all of the provided [tokens] into an AST.
    pub fn new_and_parse(tokens: Vec<Token>) -> Result<Vec<Statement>, ASTError> {
        let mut parser = ASTParser::new(tokens);
        parser.parse()
    }

    /// Attempts to parse the [tokens] within this [ASTParser] into an AST.
    pub fn parse(&mut self) -> Result<Vec<Statement>, ASTError> {
        let mut statements: Vec<Statement> = vec![];

        while let Some(token) = self.peek() {
            let statement: Statement = match token.kind {
                TokenKind::Keyword(Keyword::Func) => self.parse_function_declaration()?,
                _ => return Err(ASTErrorKind::UnexpectedToken(token.kind.clone()).at(token.span)),
            };

            statements.push(statement);
        }

        Ok(statements)
    }

    /// Attempts to parse a function declaration from the [ASTParser]'s current position.
    fn parse_function_declaration(&mut self) -> Result<Statement, ASTError> {
        // All functions must start with the func keyword.
        let func_keyword_span = self.expect_span(TokenKind::Keyword(Keyword::Func))?;

        // Then, the name of the function must be present.
        let (function_name, _) = self.expect_identifier()?;

        // Then parenthesis must surround the parameters to the function.
        self.expect(TokenKind::OpenParen)?;
        let closing_paren_span = self.expect_span(TokenKind::CloseParen)?;

        // And braces must surround the body of the function.
        self.expect(TokenKind::OpenBrace)?;
        self.expect(TokenKind::CloseBrace)?;

        Ok(Statement::new(
            FunctionDeclaration { name: function_name }.into(),
            Span::between(func_keyword_span, closing_paren_span),
        ))
    }

    /// Returns the token at the [ASTParser]'s current position.
    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.cursor)
    }

    /// Returns the token at the [ASTParser]'s current position, advancing the cursor.
    fn consume(&mut self) -> Option<&Token> {
        self.tokens.get(self.cursor).inspect(|_| self.cursor += 1)
    }

    /// Expects a token to be at the [ASTParser]'s current position, advancing the cursor.
    /// An [ASTErrorKind::UnexpectedEndOfFile] will be returned if there are no tokens left in the stream.
    fn expect_any(&mut self) -> Result<&Token, ASTError> {
        let last_token_span = self.tokens.last().map(|it| it.span).unwrap_or_default();
        self.consume().ok_or(ASTErrorKind::UnexpectedEndOfFile.at(last_token_span))
    }

    /// Expects a certain token to be at the [ASTParser]'s current position, advancing the cursor.
    fn expect(&mut self, kind: TokenKind) -> Result<&Token, ASTError> {
        let token = self.expect_any()?;
        if token.kind != kind {
            return Err(ASTErrorKind::ExpectedToken { expected: kind, got: token.kind.clone() }.at(token.span));
        }

        Ok(token)
    }

    /// Like [expect], but only returns the matched token's span.
    fn expect_span(&mut self, kind: TokenKind) -> Result<Span, ASTError> {
        self.expect(kind).map(|it| it.span)
    }

    /// Expects an identifier token to be at the [ASTParser]'s current position, advancing the cursor.
    fn expect_identifier(&mut self) -> Result<(String, Span), ASTError> {
        let token = self.expect_any()?;
        match &token.kind {
            TokenKind::Identifier(identifier) => Ok((identifier.into(), token.span)),
            _ => Err(ASTErrorKind::ExpectedIdentifier.at(token.span)),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        ast::statement::function_declaration::FunctionDeclaration,
        core::span::Span,
        lexer::token::{Keyword, TokenKind},
    };

    use super::*;
    use pretty_assertions::assert_eq;

    fn assert_ast_statements(tokens: Vec<Token>, statements: Vec<Statement>) {
        assert_eq!(ASTParser::new_and_parse(tokens), Ok(statements));
    }

    fn assert_ast_error(tokens: Vec<Token>, error: ASTError) {
        assert_eq!(ASTParser::new_and_parse(tokens), Err(error))
    }

    #[test]
    fn parse_function_declaration() {
        assert_ast_statements(
            vec![
                Token::new(TokenKind::Keyword(Keyword::Func), Span { start: 0, length: 4 }),
                Token::new(TokenKind::Identifier("main".into()), Span { start: 5, length: 4 }),
                Token::new(TokenKind::OpenParen, Span { start: 10, length: 1 }),
                Token::new(TokenKind::CloseParen, Span { start: 11, length: 1 }),
                Token::new(TokenKind::OpenBrace, Span { start: 12, length: 1 }),
                Token::new(TokenKind::CloseBrace, Span { start: 13, length: 1 }),
            ],
            vec![Statement::from(FunctionDeclaration::new("main".into()), Span { start: 0, length: 12 })],
        );
    }

    #[test]
    fn error_at_unexpected_end_of_file() {
        assert_ast_error(
            vec![
                Token::new(TokenKind::Keyword(Keyword::Func), Span { start: 0, length: 4 }),
                Token::new(TokenKind::Identifier("main".into()), Span { start: 5, length: 4 }),
            ],
            ASTErrorKind::UnexpectedEndOfFile.at(Span { start: 5, length: 4 }),
        );
    }
}
