use error::ASTError;
use node::Node;

use crate::lexer::token::{Token, TokenKind};
use std::{iter::Peekable, slice::Iter};

pub mod error;
pub mod node;

pub struct AST<'a> {
    tokens: Peekable<Iter<'a, Token>>,
}

impl<'a> AST<'a> {
    pub fn new(tokens: &'a Vec<Token>) -> AST<'a> {
        AST {
            tokens: tokens.iter().peekable(),
        }
    }

    pub fn parse(&mut self) -> Result<Vec<Node>, ASTError> {
        let mut nodes = vec![];

        while let Some(_) = self.tokens.peek() {
            nodes.push(self.parse_statement()?);
        }

        Ok(nodes)
    }

    // Attempts to parse a statement from the token stream.
    fn parse_statement(&mut self) -> Result<Node, ASTError> {
        while let Some(token) = self.tokens.peek() {
            match &token.kind {
                TokenKind::Keyword(keyword) if keyword == "func" => {
                    return self.parse_function_definition();
                }

                _ => return Err(ASTError::unexpected_token((*token).clone())),
            }
        }

        Err(ASTError::unexpected_end_of_file())
    }

    // Attempts to parse a function definition from the token stream.
    fn parse_function_definition(&mut self) -> Result<Node, ASTError> {
        // All functions must start with the `func` keyword.
        self.expect(TokenKind::Keyword("func".to_owned()))?;

        // The function name should come after the `func` keyword.
        let function_name = self.expect_identifier()?;

        // Then, the function's parameters surrounded by parenthesis.
        self.expect(TokenKind::OpenParenthesis)?;

        // TODO: Parse parameters.

        self.expect(TokenKind::CloseParenthesis)?;

        // Then, optionally the function's return type.
        let mut return_type = Option::None;
        match self.expect(TokenKind::Minus) {
            Ok(_) => {
                self.expect(TokenKind::GreaterThan)?;

                // After ->, there must be an identifier for the return type.
                return_type = Some(self.expect_identifier()?);
            }

            _ => {}
        }

        // Then, the function's body, surrounded by braces.
        self.expect(TokenKind::OpenBrace)?;

        // TODO: Parse body.

        self.expect(TokenKind::CloseBrace)?;

        return Ok(Node::function_definition(
            function_name.to_owned(),
            return_type.map(|it| it.to_owned()),
        ));
    }

    // Expects a certain token kind to be at the position in the token stream.
    fn expect(&mut self, kind: TokenKind) -> Result<&'a Token, ASTError> {
        let token = match self.tokens.peek().cloned() {
            Some(value) => value,
            None => return Err(ASTError::expected_token(kind, None)),
        };

        if token.kind == kind {
            // The token matches, we can advance the iterator.
            self.tokens.next();

            Ok(token)
        } else {
            Err(ASTError::expected_token(kind, Some((*token).clone())))
        }
    }

    // Expects an identifier to be at the position in the token stream.
    fn expect_identifier(&mut self) -> Result<&'a str, ASTError> {
        let token = match self.tokens.next() {
            Some(value) => value,
            None => return Err(ASTError::unexpected_end_of_file()),
        };

        match &token.kind {
            TokenKind::Identifier(identifier) => return Ok(identifier),
            _ => return Err(ASTError::unexpected_token(token.clone())),
        }
    }
}
