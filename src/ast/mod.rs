use crate::{
    core::{location::Location, stream::Stream},
    lexer::token::{Keyword, Token, TokenKind},
    typechecker::r#type::{Type, kind::TypeKind},
};
use error::ASTError;
use node::{
    Node,
    expression::{BinaryOperation, Expression, FunctionCall, IdentifierReference, IntegerLiteral},
    extra::FunctionParameter,
    operator::Operation,
    statement::{FunctionDefinition, Return, Statement, VariableDeclaration, VariableReassignment},
};

pub mod error;
pub mod node;

type AstResult<T> = Result<T, ASTError>;

pub struct Ast {
    tokens: Stream<Token>,
}

impl Ast {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens: Stream::new(tokens),
        }
    }

    pub fn parse(&mut self) -> AstResult<Vec<Statement>> {
        let mut nodes = vec![];

        while self.tokens.has_elements() {
            nodes.push(self.parse_statement()?);
        }

        Ok(nodes)
    }

    // Attempts to parse a statement from the token stream.
    fn parse_statement(&mut self) -> AstResult<Statement> {
        let token = self.tokens.peek().ok_or(ASTError::unexpected_end_of_file())?;

        let node = match &token.kind {
            TokenKind::Keyword(keyword) => match keyword {
                // A function definition should not end in a semicolon, hence the reason for `return` here.
                Keyword::Func => return self.parse_function_definition(),
                Keyword::Return => self.parse_return_statement()?,
            },

            TokenKind::Identifier(_) => {
                if self.after_next_is(TokenKind::Equals) {
                    self.parse_variable_reassignment()?
                } else {
                    self.parse_variable_declaration()?
                }
            }

            _ => return Err(ASTError::unexpected_token((*token).clone())),
        };

        // If the code reaches here, it means that the statement should end in a semicolon.
        self.expect(TokenKind::Semicolon)?;

        Ok(node)
    }

    // Attempts to parse an expression from the token stream.
    fn parse_expression(&mut self) -> AstResult<Expression> {
        self.parse_addition_subtraction_expression()
    }

    fn parse_addition_subtraction_expression(&mut self) -> AstResult<Expression> {
        let left = self.parse_multiplication_division_expression()?;

        if self.next_is(TokenKind::Plus) || self.next_is(TokenKind::Minus) {
            // It's safe to unwrap here, `next_is` would return `false` if there was no token at the next position.
            let operator_token = self.tokens.next().unwrap().clone();

            // We're only dealing with addition and subtraction in this branch.
            let operation = match operator_token.kind {
                TokenKind::Plus => Operation::Add,
                _ => Operation::Subtract,
            };

            let right = self.parse_expression()?;

            return Ok(Expression::BinaryOperation(BinaryOperation {
                node: Node::new(operator_token.location),
                operation,
                left: Box::new(left),
                right: Box::new(right),
                expected_type: None,
            }));
        }

        Ok(left)
    }

    fn parse_multiplication_division_expression(&mut self) -> AstResult<Expression> {
        let left = self.parse_value()?;

        if self.next_is(TokenKind::Asterisk) || self.next_is(TokenKind::Slash) {
            // It's safe to unwrap here, `next_is` would return `false` if there was no token at the next position.
            let operator_token = self.tokens.next().unwrap().clone();

            // We're only dealing with multiplication and division in this branch.
            let operation = match operator_token.kind {
                TokenKind::Asterisk => Operation::Multiply,
                _ => Operation::Divide,
            };

            let right = self.parse_expression()?;

            return Ok(Expression::BinaryOperation(BinaryOperation {
                node: Node::new(operator_token.location),
                operation,
                left: Box::new(left),
                right: Box::new(right),
                expected_type: None,
            }));
        }

        Ok(left)
    }

    fn parse_value(&mut self) -> AstResult<Expression> {
        let token = self.tokens.next().ok_or(ASTError::unexpected_end_of_file()).cloned()?;
        let node = Node::new(token.location);

        match &token.kind {
            TokenKind::IntegerLiteral(value) => Ok(Expression::IntegerLiteral(IntegerLiteral {
                node,
                value: *value,
                expected_type: None,
            })),

            TokenKind::Identifier(name) => {
                // If the next token is an opening parenthesis, this is most likely a function call.
                if self.next_is(TokenKind::OpenParenthesis) {
                    self.parse_function_call(name, token.location)
                } else {
                    Ok(Expression::IdentifierReference(IdentifierReference {
                        node,
                        name: name.clone(),
                        expected_type: None,
                    }))
                }
            }

            _ => Err(ASTError::unexpected_token(token)),
        }
    }

    // Attempts to parse a function call from the token stream.
    fn parse_function_call(&mut self, name: &str, location: Location) -> AstResult<Expression> {
        self.expect(TokenKind::OpenParenthesis)?;

        let mut arguments = Vec::new();

        if self.expect(TokenKind::CloseParenthesis).is_err() {
            // If the next token is not a closing parenthesis, we must parse the function call's arguments.
            loop {
                arguments.push(self.parse_expression()?);

                // If we reach a closing parenthesis, we have finished parsing the arguments.
                if self.expect(TokenKind::CloseParenthesis).is_ok() {
                    break;
                } else {
                    // Otherwise, we must expect a comma.
                    self.expect(TokenKind::Comma)?;
                }
            }
        }

        Ok(Expression::FunctionCall(FunctionCall {
            node: Node::new(location),
            name: name.to_owned(),
            arguments,
            expected_type: None,
        }))
    }

    // Attempts to parse a function definition from the token stream.
    fn parse_function_definition(&mut self) -> AstResult<Statement> {
        // All functions must start with the `func` keyword.
        self.expect(TokenKind::Keyword(Keyword::Func))?;

        // The function name should come after the `func` keyword.
        let (function_name, function_name_location) = self.expect_identifier()?;

        // Then, the function's parameters surrounded by parenthesis.
        self.expect(TokenKind::OpenParenthesis)?;

        let mut parameters = Vec::new();

        if self.expect(TokenKind::CloseParenthesis).is_err() {
            // If the next token is not a closing parenthesis, we must parse the function's parameters.
            loop {
                // The first part of the parameter is the identifier.
                let (parameter_name, parameter_location) = self.expect_identifier()?;

                // The next part must be a colon.
                self.expect(TokenKind::Colon)?;

                // Then, the type of the parameter.
                let (parameter_type, parameter_type_location) = self.expect_identifier()?;

                parameters.push(FunctionParameter::new(
                    parameter_name,
                    Type::new(TypeKind::Unresolved(parameter_type), parameter_type_location),
                    parameter_location,
                ));

                // If we reach a closing parenthesis, we have finished parsing the parameters.
                if self.expect(TokenKind::CloseParenthesis).is_ok() {
                    break;
                } else {
                    // Otherwise, we must expect a comma.
                    self.expect(TokenKind::Comma)?;
                }
            }
        }

        // Then, optionally the function's return type.
        let mut return_type = Option::None;

        if self.expect(TokenKind::Minus).is_ok() {
            self.expect(TokenKind::GreaterThan)?;

            // After ->, there must be an identifier for the return type.
            return_type = self
                .expect_identifier()
                .ok()
                .map(|(name, location)| Type::new(TypeKind::Unresolved(name.to_owned()), location))
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

        Ok(Statement::FunctionDefinition(FunctionDefinition {
            node: Node::new(function_name_location),
            name: function_name,
            parameters,
            return_type,
            body,
        }))
    }

    // Attempts to parse a variable declaration from the token stream.
    fn parse_variable_declaration(&mut self) -> AstResult<Statement> {
        let (type_name, type_location) = self.expect_identifier()?;
        let (name, name_location) = self.expect_identifier()?;

        self.expect(TokenKind::Equals)?;

        let value = self.parse_expression()?;

        Ok(Statement::VariableDeclaration(VariableDeclaration {
            node: Node::new(name_location),
            declared_type: Type::new(TypeKind::Unresolved(type_name), type_location),
            name,
            value,
        }))
    }

    // Attempts to parse a variable re-assignment from the token stream.
    fn parse_variable_reassignment(&mut self) -> AstResult<Statement> {
        let (name, name_location) = self.expect_identifier()?;
        self.expect(TokenKind::Equals)?;

        let value = self.parse_expression()?;

        Ok(Statement::VariableReassignment(VariableReassignment {
            node: Node::new(name_location),
            name,
            value,
        }))
    }

    // Attempts to parse a return statement from the token stream.
    pub fn parse_return_statement(&mut self) -> AstResult<Statement> {
        // All return statements must start with the `func` keyword.
        let return_token = self.expect(TokenKind::Keyword(Keyword::Return))?;

        // If the next token is not a semicolon, it has an associated value.
        let mut value = None;
        if !self.next_is(TokenKind::Semicolon) {
            value = Some(self.parse_expression()?);
        }

        Ok(Statement::Return(Return {
            node: Node::new(return_token.location),
            value,
        }))
    }

    fn next_is(&self, kind: TokenKind) -> bool {
        let token = match self.tokens.peek() {
            Some(value) => value,
            None => return false,
        };

        token.kind == kind
    }

    fn after_next_is(&self, kind: TokenKind) -> bool {
        let token = match self.tokens.peek_at(1) {
            Some(value) => value,
            None => return false,
        };

        token.kind == kind
    }

    // Expects a certain token kind to be at the position in the token stream.
    fn expect(&mut self, kind: TokenKind) -> AstResult<Token> {
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
    fn expect_identifier(&mut self) -> AstResult<(String, Location)> {
        let token = self.tokens.next().ok_or(ASTError::unexpected_end_of_file())?;

        match &token.kind {
            TokenKind::Identifier(identifier) => Ok((identifier.clone(), token.location)),
            _ => Err(ASTError::unexpected_token(token.clone())),
        }
    }
}
