use crate::{
    ast::{
        error::{ASTError, ASTErrorKind},
        statement::Statement,
    },
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
        todo!()
    }

    /// Returns the token at the [ASTParser]'s current position.
    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.cursor)
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
            vec![Statement::from(FunctionDeclaration::new("main".into()), Span { start: 0, length: 4 })],
        );
    }
}
