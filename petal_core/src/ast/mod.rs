use crate::{
    core::{location::Location, stream::Stream},
    lexer::token::{Keyword, Token, TokenKind},
    typechecker::r#type::{Type, kind::TypeKind},
};
use error::ASTError;
use node::{
    Node,
    expression::{
        BinaryComparison, BinaryOperation, BooleanLiteral, Expression, FunctionCall, IdentifierReference,
        IntegerLiteral, StringLiteral,
    },
    extra::FunctionParameter,
    operator::{Comparison, Operation},
    statement::{FunctionDefinition, If, Return, Statement, VariableDeclaration, VariableReassignment},
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
        let token = self.tokens.peek().cloned().ok_or(ASTError::unexpected_end_of_file())?;

        let node = match &token.kind {
            TokenKind::Keyword(keyword) => match keyword {
                // A function definition should not end in a semicolon, hence the reason for `return` here.
                Keyword::Func | Keyword::Extern => return self.parse_function_definition(),
                Keyword::Return => self.parse_return_statement()?,
                Keyword::If => return self.parse_if_statement(),

                Keyword::Else => return Err(ASTError::dangling_else(token.location)),
                Keyword::True | Keyword::False => return Err(ASTError::unexpected_token(token)),
            },

            TokenKind::Identifier(name) => {
                if self.after_next_is(TokenKind::Equals) {
                    self.parse_variable_reassignment()?
                } else if self.after_next_is(TokenKind::OpenParenthesis) {
                    self.tokens.next();

                    Statement::FunctionCall(self.parse_function_call(name, token.location)?)
                } else {
                    self.parse_variable_declaration()?
                }
            }

            TokenKind::Ampersand => self.parse_variable_declaration()?,

            _ => return Err(ASTError::unexpected_token(token)),
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
        let left = self.parse_less_than_greater_than()?;

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

    fn parse_less_than_greater_than(&mut self) -> AstResult<Expression> {
        let left = self.parse_value()?;

        if self.next_is(TokenKind::LessThan) || self.next_is(TokenKind::GreaterThan) {
            // It's safe to unwrap here, `next_is` would return `false` if there was no token at the next position.
            let operator_token = self.tokens.next().unwrap().clone();

            // We're only dealing with multiplication and division in this branch.
            let comparison = match operator_token.kind {
                TokenKind::LessThan => Comparison::LessThan,
                _ => Comparison::GreaterThan,
            };

            let right = self.parse_expression()?;

            return Ok(Expression::BinaryComparison(BinaryComparison {
                node: Node::new(operator_token.location),
                comparison,
                left: Box::new(left),
                right: Box::new(right),
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

            TokenKind::StringLiteral(value) => Ok(Expression::StringLiteral(StringLiteral {
                node,
                value: value.to_string(),
            })),

            TokenKind::Keyword(keyword) => match keyword {
                Keyword::True => Ok(Expression::BooleanLiteral(BooleanLiteral { node, value: true })),
                Keyword::False => Ok(Expression::BooleanLiteral(BooleanLiteral { node, value: false })),

                _ => Err(ASTError::unexpected_token(token)),
            },

            TokenKind::Identifier(name) => {
                // If the next token is an opening parenthesis, this is most likely a function call.
                if self.next_is(TokenKind::OpenParenthesis) {
                    Ok(Expression::FunctionCall(
                        self.parse_function_call(name, token.location)?,
                    ))
                } else {
                    Ok(Expression::IdentifierReference(IdentifierReference {
                        node,
                        name: name.clone(),
                        expected_type: None,
                        is_reference: false,
                    }))
                }
            }

            TokenKind::Ampersand => {
                // The next token shoiuld be an identifier.
                let (name, name_location) = self.expect_identifier()?;

                Ok(Expression::IdentifierReference(IdentifierReference {
                    node: Node::new(name_location),
                    name,
                    expected_type: None,
                    is_reference: true,
                }))
            }

            _ => Err(ASTError::unexpected_token(token)),
        }
    }

    // Attempts to parse a function call from the token stream.
    fn parse_function_call(&mut self, name: &str, location: Location) -> AstResult<FunctionCall> {
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

        Ok(FunctionCall {
            node: Node::new(location),
            name: name.to_owned(),
            arguments,
            expected_type: None,
        })
    }

    // Attempts to parse a function definition from the token stream.
    fn parse_function_definition(&mut self) -> AstResult<Statement> {
        // If the function starts with the `extern` keyword, the function should have no body.
        let mut is_extern = false;
        if self.next_is(TokenKind::Keyword(Keyword::Extern)) {
            self.expect(TokenKind::Keyword(Keyword::Extern))?;
            is_extern = true;
        }

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
                let parameter_type = self.expect_type()?;

                parameters.push(FunctionParameter::new(
                    parameter_name,
                    parameter_type,
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
            return_type = Some(self.expect_type()?);
        }

        let mut body = vec![];

        if is_extern {
            // An external function must not have a body.
            self.expect(TokenKind::Semicolon)?;
        } else {
            // Then, the function's body, surrounded by braces.
            self.expect(TokenKind::OpenBrace)?;

            while let Some(token) = self.tokens.peek() {
                if token.kind == TokenKind::CloseBrace {
                    break;
                }

                body.push(self.parse_statement()?);
            }

            self.expect(TokenKind::CloseBrace)?;
        }

        Ok(Statement::FunctionDefinition(FunctionDefinition {
            node: Node::new(function_name_location),
            name: function_name,
            is_extern,
            parameters,
            return_type,
            body,
        }))
    }

    // Attempts to parse a variable declaration from the token stream.
    fn parse_variable_declaration(&mut self) -> AstResult<Statement> {
        let variable_type = self.expect_type()?;
        let (name, name_location) = self.expect_identifier()?;

        self.expect(TokenKind::Equals)?;

        let value = self.parse_expression()?;

        Ok(Statement::VariableDeclaration(VariableDeclaration {
            node: Node::new(name_location),
            declared_type: variable_type,
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

    // Attempts to parse an if statement from the token stream.
    fn parse_if_statement(&mut self) -> AstResult<Statement> {
        // All if statements must start with the `if` keyword.
        let if_token = self.expect(TokenKind::Keyword(Keyword::If))?;

        // The next token(s) must be a valid expression.
        let condition = self.parse_expression()?;

        // Then, a block of code to execute if the condition was true.
        let mut body = vec![];

        self.expect(TokenKind::OpenBrace)?;

        while let Some(token) = self.tokens.peek() {
            if token.kind == TokenKind::CloseBrace {
                break;
            }

            body.push(self.parse_statement()?);
        }

        self.expect(TokenKind::CloseBrace)?;

        Ok(Statement::If(If {
            node: Node::new(if_token.location),
            condition,
            block: body,
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

    // Expects a valid type to be at the next position in the token stream.
    fn expect_type(&mut self) -> AstResult<Type> {
        // If the type starts with an ampersand, it's a reference.
        let is_reference = self.expect(TokenKind::Ampersand).is_ok();
        let (type_name, type_location) = self.expect_identifier()?;

        let mut kind = TypeKind::Unresolved(type_name);
        if is_reference {
            kind = TypeKind::Reference(Box::new(kind));
        }

        Ok(Type::new(kind, type_location))
    }
}
