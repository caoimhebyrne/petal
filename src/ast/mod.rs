use crate::{
    core::{location::Location, stream::Stream},
    lexer::token::{Token, TokenKind},
    typechecker::r#type::{Type, kind::TypeKind},
};
use error::ASTError;
use node::{
    Node,
    kind::{
        BinaryOperationNode, FunctionCallNode, FunctionDefinitionNode, IdentifierReferenceNode, IntegerLiteralNode,
        NodeKind, ReturnNode, VariableDeclarationNode,
    },
    operator::BinaryOperation,
};

pub mod error;
pub mod node;

pub struct Ast {
    tokens: Stream<Token>,
}

impl Ast {
    pub fn new(tokens: Vec<Token>) -> Ast {
        Ast {
            tokens: Stream::new(tokens),
        }
    }

    pub fn parse(&mut self) -> Result<Vec<Node>, ASTError> {
        let mut nodes = vec![];

        while self.tokens.has_elements() {
            nodes.push(self.parse_statement()?);
        }

        Ok(nodes)
    }

    // Attempts to parse a statement from the token stream.
    fn parse_statement(&mut self) -> Result<Node, ASTError> {
        let token = self.tokens.peek().ok_or(ASTError::unexpected_end_of_file())?;

        let node = match &token.kind {
            TokenKind::Keyword(keyword) => match keyword.as_str() {
                "func" => return self.parse_function_definition(),
                "return" => self.parse_return_statement()?,

                _ => return Err(ASTError::unexpected_token((*token).clone())),
            },

            TokenKind::Identifier(_) => self.parse_variable_declaration()?,

            _ => return Err(ASTError::unexpected_token((*token).clone())),
        };

        // If the code reaches here, it means that the statement should end in a semicolon.
        self.expect(TokenKind::Semicolon)?;

        Ok(node)
    }

    // Attempts to parse an expression from the token stream.
    fn parse_expression(&mut self) -> Result<Node, ASTError> {
        self.parse_addition_subtraction_expression()
    }

    fn parse_addition_subtraction_expression(&mut self) -> Result<Node, ASTError> {
        let left = self.parse_multiplication_division_expression()?;

        if self.next_is(TokenKind::Plus) || self.next_is(TokenKind::Minus) {
            // It's safe to unwrap here, `next_is` would return `false` if there was no token at the next position.
            let operator_token = self.tokens.next().unwrap().clone();

            // We're only dealing with addition and subtraction in this branch.
            let operation = match operator_token.kind {
                TokenKind::Plus => BinaryOperation::Add,
                _ => BinaryOperation::Subtract,
            };

            let right = self.parse_expression()?;

            return Ok(Node::new(
                NodeKind::BinaryOperation(BinaryOperationNode {
                    operation,
                    left: Box::new(left),
                    right: Box::new(right),
                    value_type: None,
                }),
                operator_token.location,
            ));
        }

        Ok(left)
    }

    fn parse_multiplication_division_expression(&mut self) -> Result<Node, ASTError> {
        let left = self.parse_value()?;

        if self.next_is(TokenKind::Asterisk) || self.next_is(TokenKind::Slash) {
            // It's safe to unwrap here, `next_is` would return `false` if there was no token at the next position.
            let operator_token = self.tokens.next().unwrap().clone();

            // We're only dealing with multiplication and division in this branch.
            let operation = match operator_token.kind {
                TokenKind::Asterisk => BinaryOperation::Multiply,
                _ => BinaryOperation::Divide,
            };

            let right = self.parse_expression()?;

            return Ok(Node::new(
                NodeKind::BinaryOperation(BinaryOperationNode {
                    operation,
                    left: Box::new(left),
                    right: Box::new(right),
                    value_type: None,
                }),
                operator_token.location,
            ));
        }

        Ok(left)
    }

    fn parse_value(&mut self) -> Result<Node, ASTError> {
        let token = self.tokens.next().ok_or(ASTError::unexpected_end_of_file()).cloned()?;

        match &token.kind {
            TokenKind::IntegerLiteral(value) => Ok(Node::new(
                NodeKind::IntegerLiteral(IntegerLiteralNode {
                    value: *value,
                    r#type: None,
                }),
                token.location,
            )),

            TokenKind::Identifier(name) => {
                // If the next token is an opening parenthesis, this is most likely a function call.
                if self.next_is(TokenKind::OpenParenthesis) {
                    self.parse_function_call(name, token.location)
                } else {
                    Ok(Node::new(
                        NodeKind::IdentifierReference(IdentifierReferenceNode {
                            name: name.to_string(),
                            r#type: None,
                        }),
                        token.location,
                    ))
                }
            }

            _ => Err(ASTError::unexpected_token(token)),
        }
    }

    // Attempts to parse a function call from the token stream.
    fn parse_function_call(&mut self, name: &str, location: Location) -> Result<Node, ASTError> {
        self.expect(TokenKind::OpenParenthesis)?;

        // TODO: Parse a function call's arguments.

        self.expect(TokenKind::CloseParenthesis)?;

        Ok(Node::new(
            NodeKind::FunctionCall(FunctionCallNode {
                name: name.to_string(),
                return_type: None,
            }),
            location,
        ))
    }

    // Attempts to parse a function definition from the token stream.
    fn parse_function_definition(&mut self) -> Result<Node, ASTError> {
        // All functions must start with the `func` keyword.
        self.expect(TokenKind::Keyword("func".to_owned()))?;

        // The function name should come after the `func` keyword.
        let (function_name, function_name_location) = self.expect_identifier()?;

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
                return_type = self
                    .expect_identifier()
                    .ok()
                    .map(|(name, location)| Type::new(TypeKind::Unresolved(name.to_owned()), Some(location)))
            }

            _ => {}
        }

        // Then, the function's body, surrounded by braces.
        self.expect(TokenKind::OpenBrace)?;

        let mut body = vec![];

        while let Some(token) = self.tokens.peek() {
            if token.kind == TokenKind::CloseBrace {
                break;
            }

            body.push(self.parse_statement()?);
        }

        self.expect(TokenKind::CloseBrace)?;

        Ok(Node::new(
            NodeKind::FunctionDefinition(FunctionDefinitionNode {
                name: function_name.to_string(),
                return_type: return_type,
                body,
            }),
            function_name_location,
        ))
    }

    // Attempts to parse a variable declaration from the token stream.
    fn parse_variable_declaration(&mut self) -> Result<Node, ASTError> {
        let (type_name, type_location) = self.expect_identifier()?;
        let (name, name_location) = self.expect_identifier()?;

        self.expect(TokenKind::Equals)?;

        let value = self.parse_expression()?;

        Ok(Node::new(
            NodeKind::VariableDeclaration(VariableDeclarationNode {
                name: name.to_string(),
                declared_type: Type::new(TypeKind::Unresolved(type_name.to_string()), Some(type_location)),
                value: Box::new(value),
            }),
            name_location,
        ))
    }

    // Attempts to parse a return statement from the token stream.
    pub fn parse_return_statement(&mut self) -> Result<Node, ASTError> {
        // All return statements must start with the `func` keyword.
        let return_token = self.expect(TokenKind::Keyword("return".to_owned()))?;

        // If the next token is not a semicolon, it has an associated value.
        let mut value = None;
        if !self.next_is(TokenKind::Semicolon) {
            value = Some(Box::new(self.parse_expression()?));
        }

        Ok(Node::new(NodeKind::Return(ReturnNode { value }), return_token.location))
    }

    fn next_is(&self, kind: TokenKind) -> bool {
        let token = match self.tokens.peek() {
            Some(value) => value,
            None => return false,
        };

        return token.kind == kind;
    }

    // Expects a certain token kind to be at the position in the token stream.
    fn expect(&mut self, kind: TokenKind) -> Result<Token, ASTError> {
        let token = match self.tokens.peek().cloned() {
            Some(value) => value,
            None => return Err(ASTError::expected_token(kind, None)),
        };

        if token.kind != kind {
            return Err(ASTError::expected_token(kind, Some(token.clone())));
        }

        // The token matches, we can advance the iterator.
        self.tokens.advance_by(1);

        Ok(token)
    }

    // Expects an identifier to be at the position in the token stream.
    fn expect_identifier(&mut self) -> Result<(String, Location), ASTError> {
        let token = self.tokens.next().ok_or(ASTError::unexpected_end_of_file())?;

        match &token.kind {
            TokenKind::Identifier(identifier) => Ok((identifier.clone(), token.location)),
            _ => Err(ASTError::unexpected_token(token.clone())),
        }
    }
}
