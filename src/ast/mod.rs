use crate::{
    ast::{
        error::{
            ASTError,
            ASTErrorKind,
        },
        expression::{
            Expression,
            ExpressionKind,
            binary_operation::{
                BinaryOperand,
                BinaryOperation,
            },
            function_call::FunctionCall,
        },
        statement::{
            Statement,
            function_declaration::FunctionDeclaration,
            r#return::Return,
            variable_assignment::VariableAssignment,
            variable_declaration::VariableDeclaration,
        },
        type_expr::TypeExpr,
    },
    core::span::Span,
    lexer::token::{
        Keyword,
        Token,
        TokenKind,
    },
    typechecker::r#type::Type,
};

pub mod error;
pub mod expression;
pub mod statement;
pub mod type_expr;

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

    /// Attempts to parse a statement at the [ASTParser]'s current position.
    fn parse_statement(&mut self) -> Result<Statement, ASTError> {
        let token = self.peek_expect_any()?;

        let statement = match token.kind {
            TokenKind::Keyword(Keyword::Return) => self.parse_return()?,

            TokenKind::Identifier(_) => {
                if self.peek_nth(1).map(|it| it.kind == TokenKind::Equals).unwrap_or_default() {
                    self.parse_variable_assignment()
                } else {
                    self.parse_variable_declaration()
                }?
            }

            _ => return Err(ASTErrorKind::ExpectedStatement(token.kind.clone()).at(token.span)),
        };

        self.expect(TokenKind::Semicolon)?;

        Ok(statement)
    }

    /// Attempts to parse an expression at the [ASTParser]'s current position.
    fn parse_expression(&mut self) -> Result<Expression, ASTError> {
        self.parse_addition_or_subtraction_expression()
    }

    /// Attempts to parse an addition or subtraction expression at the [ASTParser]'s current position.
    fn parse_addition_or_subtraction_expression(&mut self) -> Result<Expression, ASTError> {
        let left = self.parse_multiplication_or_division_expression()?;

        let expression = if self.peek_is(TokenKind::Plus) {
            self.expect(TokenKind::Plus)?;

            let right = self.parse_expression()?;
            let span = Span::between(left.span, right.span);

            Expression::new(BinaryOperation::new(left, right, BinaryOperand::Add).into(), span)
        } else if self.peek_is(TokenKind::Hyphen) {
            self.expect(TokenKind::Hyphen)?;

            let right = self.parse_expression()?;
            let span = Span::between(left.span, right.span);

            Expression::new(BinaryOperation::new(left, right, BinaryOperand::Subtract).into(), span)
        } else {
            left
        };

        Ok(expression)
    }

    /// Attempts to parse a multiplication or division expression at the [ASTParser]'s current position.
    fn parse_multiplication_or_division_expression(&mut self) -> Result<Expression, ASTError> {
        let left = self.parse_value()?;

        let expression = if self.peek_is(TokenKind::Asterisk) {
            self.expect(TokenKind::Asterisk)?;

            let right = self.parse_expression()?;
            let span = Span::between(left.span, right.span);

            Expression::new(BinaryOperation::new(left, right, BinaryOperand::Multiply).into(), span)
        } else if self.peek_is(TokenKind::ForwardSlash) {
            self.expect(TokenKind::ForwardSlash)?;

            let right = self.parse_expression()?;
            let span = Span::between(left.span, right.span);

            Expression::new(BinaryOperation::new(left, right, BinaryOperand::Divide).into(), span)
        } else {
            left
        };

        Ok(expression)
    }

    /// Attempts to parse a simple value at the [ASTParser]'s current position.
    fn parse_value(&mut self) -> Result<Expression, ASTError> {
        let token = self.peek_expect_any()?;

        let expression = match &token.kind {
            TokenKind::Number(value) => {
                let value = *value;
                let span = token.span;
                self.consume();
                Expression::new(ExpressionKind::NumberLiteral(value), span)
            }

            TokenKind::Keyword(Keyword::True) => {
                let span = token.span;
                self.consume();
                Expression::new(ExpressionKind::BooleanLiteral(true), span)
            }

            TokenKind::Keyword(Keyword::False) => {
                let span = token.span;
                self.consume();
                Expression::new(ExpressionKind::BooleanLiteral(false), span)
            }

            TokenKind::Identifier(name) => {
                if self.peek_nth(1).map(|it| it.kind == TokenKind::OpenParen).unwrap_or_default() {
                    let (function_call, span) = self.parse_function_call()?;
                    Expression::new(function_call.into(), span)
                } else {
                    // FIXME: We need to copy the span before attempting to acquire a mutable reference via consume.
                    let name = name.clone();
                    let span = token.span;
                    self.consume();
                    Expression::new(ExpressionKind::IdentifierReference(name), span)
                }
            }

            _ => return Err(ASTErrorKind::ExpectedExpression(token.kind.clone()).at(token.span)),
        };

        Ok(expression)
    }

    /// Attempts to parse a function declaration from the [ASTParser]'s current position.
    fn parse_function_declaration(&mut self) -> Result<Statement, ASTError> {
        // All functions must start with the func keyword.
        let func_keyword_span = self.expect_span(TokenKind::Keyword(Keyword::Func))?;

        // Then, the name of the function must be present.
        let (function_name, _) = self.expect_identifier()?;

        let mut builder = FunctionDeclaration::builder(function_name);

        // Then parenthesis must surround the parameters to the function.
        self.expect(TokenKind::OpenParen)?;

        while !self.peek_is(TokenKind::CloseParen) {
            let is_named = if self.peek_is(TokenKind::Tilda) {
                self.expect(TokenKind::Tilda)?;
                true
            } else {
                false
            };

            let (parameter_name, parameter_name_span) = self.expect_identifier()?;
            self.expect(TokenKind::Colon)?;
            let (parameter_type_name, parameter_type_name_span) = self.expect_identifier()?;

            builder = builder.parameter(
                parameter_name,
                TypeExpr::Named(parameter_type_name),
                Type::default(),
                is_named,
                Span::between(parameter_name_span, parameter_type_name_span),
            );

            if self.peek_is(TokenKind::CloseParen) {
                continue;
            }

            self.expect(TokenKind::Comma)?;
        }

        self.expect(TokenKind::CloseParen)?;

        // There may be a `->` token, indicating that an explicit return type is being used.
        if self.peek_is(TokenKind::Hyphen) {
            self.expect(TokenKind::Hyphen)?;
            self.expect(TokenKind::RightAngleBracket)?;

            let (return_type_name, _) = self.expect_identifier()?;

            builder = builder.return_type(TypeExpr::named(return_type_name), Type::Unknown);
        }

        // And braces must surround the body of the function.
        self.expect(TokenKind::OpenBrace)?;

        while !self.peek_is(TokenKind::CloseBrace) {
            builder = builder.statement(self.parse_statement()?);
        }

        let closing_brace_span = self.expect_span(TokenKind::CloseBrace)?;

        Ok(Statement::new(builder.build().into(), Span::between(func_keyword_span, closing_brace_span)))
    }

    /// Attempts to parse a return statement from the [ASTParser]'s current position.
    fn parse_return(&mut self) -> Result<Statement, ASTError> {
        let return_keyword_span = self.expect_span(TokenKind::Keyword(Keyword::Return))?;

        if self.peek_is(TokenKind::Semicolon) {
            return Ok(Statement::from(Return::new(None), return_keyword_span));
        }

        let value = self.parse_expression()?;
        let span = Span::between(return_keyword_span, value.span);
        Ok(Statement::from(Return::new(Some(value)), span))
    }

    /// Attempts to parse a variable declaration statement from the [ASTParser]'s current position.
    fn parse_variable_declaration(&mut self) -> Result<Statement, ASTError> {
        // The first token must be the type of the variable.
        let (type_name, type_span) = self.expect_identifier()?;

        // The next token must be the name of the variable.
        let (name, _) = self.expect_identifier()?;

        // The next token must be an equals.
        self.expect(TokenKind::Equals)?;

        // And finally, there must be an expression.
        let value = self.parse_expression()?;

        let span = Span::between(type_span, value.span);
        Ok(Statement::from(VariableDeclaration::new(name, TypeExpr::named(type_name), Type::Unknown, value), span))
    }

    /// Attempts to parse a variable assignment statement from the [ASTParser]'s current position.
    fn parse_variable_assignment(&mut self) -> Result<Statement, ASTError> {
        // The first token must be the name of the variable.
        let (name, name_span) = self.expect_identifier()?;

        // The next token must be an equals.
        self.expect(TokenKind::Equals)?;

        // And finally, there must be an expression.
        let value = self.parse_expression()?;

        let span = Span::between(name_span, value.span);
        Ok(Statement::from(VariableAssignment::new(name, value), span))
    }

    /// Attempts to parse a function call from the [ASTParser]'s current position.
    fn parse_function_call(&mut self) -> Result<(FunctionCall, Span), ASTError> {
        // The first token must be the name of the function.
        let (function_name, function_name_span) = self.expect_identifier()?;

        let mut builder = FunctionCall::builder(function_name);

        // Then, the arguments of the function will be surrounded by parenthesis.
        self.expect(TokenKind::OpenParen)?;

        while !self.peek_is(TokenKind::CloseParen) {
            // If the token after the current one is present, then this is a named parameter.
            let identifier = if self.peek_nth(1).map(|it| it.kind == TokenKind::Colon).unwrap_or_default() {
                let identifier = self.expect_identifier()?;

                self.expect(TokenKind::Colon)?;

                Some(identifier)
            } else {
                None
            };

            let value = self.parse_expression()?;
            let span = identifier.as_ref().map(|it| Span::between(it.1, value.span)).unwrap_or(value.span);

            builder = builder.argument(identifier.map(|it| it.0), value, span);

            if self.peek_is(TokenKind::CloseParen) {
                continue;
            }

            self.expect(TokenKind::Comma)?;
        }

        let close_paren_span = self.expect_span(TokenKind::CloseParen)?;

        Ok((builder.build(), Span::between(function_name_span, close_paren_span)))
    }

    /// Returns the token at the [ASTParser]'s current position.
    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.cursor)
    }

    /// Returns the token at the [ASTParser]'s current position.
    fn peek_nth(&self, offset: usize) -> Option<&Token> {
        self.tokens.get(self.cursor + offset)
    }

    /// Returns whether the token at the [ASTParser]'s current position is of a certain [TokenKind].
    fn peek_is(&self, kind: TokenKind) -> bool {
        self.peek().map(|it| it.kind == kind).unwrap_or_default()
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

    /// Expects a token to be at the [ASTParser]'s current position, advancing the cursor.
    /// An [ASTErrorKind::UnexpectedEndOfFile] will be returned if there are no tokens left in the stream.
    fn peek_expect_any(&self) -> Result<&Token, ASTError> {
        let last_token_span = self.tokens.last().map(|it| it.span).unwrap_or_default();
        self.peek().ok_or(ASTErrorKind::UnexpectedEndOfFile.at(last_token_span))
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
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::{
        core::span::Span,
        lexer::token::{
            Keyword,
            TokenKind,
        },
    };

    fn assert_ast_error(tokens: Vec<Token>, error: ASTError) {
        assert_eq!(ASTParser::new_and_parse(tokens), Err(error))
    }

    #[test]
    fn parse_function_declaration() {
        insta::assert_debug_snapshot!(ASTParser::new_and_parse(vec![
            Token::new(TokenKind::Keyword(Keyword::Func), Span { start: 0, length: 4 }),
            Token::new(TokenKind::Identifier("main".into()), Span { start: 5, length: 4 }),
            Token::new(TokenKind::OpenParen, Span { start: 10, length: 1 }),
            Token::new(TokenKind::CloseParen, Span { start: 11, length: 1 }),
            Token::new(TokenKind::OpenBrace, Span { start: 12, length: 1 }),
            Token::new(TokenKind::CloseBrace, Span { start: 13, length: 1 }),
        ]), @r#"
        Ok(
            [
                Statement {
                    kind: FunctionDeclaration(
                        FunctionDeclaration {
                            name: "main",
                            body: [],
                            parameters: [],
                            return_type_expr: None,
                            return_type: Unknown,
                        },
                    ),
                    span: Span {
                        start: 0,
                        length: 14,
                    },
                },
            ],
        )
        "#);
    }

    #[test]
    fn parse_function_declaration_with_parameter() {
        insta::assert_debug_snapshot!(ASTParser::new_and_parse(vec![
            Token::new(TokenKind::Keyword(Keyword::Func), Span { start: 0, length: 4 }),
            Token::new(TokenKind::Identifier("main".into()), Span { start: 5, length: 4 }),
            Token::new(TokenKind::OpenParen, Span { start: 10, length: 1 }),
            Token::new(TokenKind::Identifier("argc".into()), Span { start: 11, length: 4 }),
            Token::new(TokenKind::Colon, Span { start: 15, length: 1 }),
            Token::new(TokenKind::Identifier("i32".into()), Span { start: 16, length: 3 }),
            Token::new(TokenKind::CloseParen, Span { start: 19, length: 1 }),
            Token::new(TokenKind::OpenBrace, Span { start: 20, length: 1 }),
            Token::new(TokenKind::CloseBrace, Span { start: 21, length: 1 }),
        ]), @r#"
        Ok(
            [
                Statement {
                    kind: FunctionDeclaration(
                        FunctionDeclaration {
                            name: "main",
                            body: [],
                            parameters: [
                                FunctionParameter {
                                    name: "argc",
                                    type_expr: Named(
                                        "i32",
                                    ),
                                    type: Unknown,
                                    is_named: false,
                                    span: Span {
                                        start: 11,
                                        length: 8,
                                    },
                                },
                            ],
                            return_type_expr: None,
                            return_type: Unknown,
                        },
                    ),
                    span: Span {
                        start: 0,
                        length: 22,
                    },
                },
            ],
        )
        "#);
    }

    #[test]
    fn parse_function_declaration_with_parameters() {
        insta::assert_debug_snapshot!(ASTParser::new_and_parse(vec![
            Token::new(TokenKind::Keyword(Keyword::Func), Span { start: 0, length: 4 }),
            Token::new(TokenKind::Identifier("main".into()), Span { start: 5, length: 4 }),
            Token::new(TokenKind::OpenParen, Span { start: 10, length: 1 }),

            Token::new(TokenKind::Identifier("argc".into()), Span { start: 11, length: 4 }),
            Token::new(TokenKind::Colon, Span { start: 15, length: 1 }),
            Token::new(TokenKind::Identifier("i32".into()), Span { start: 16, length: 3 }),

            Token::new(TokenKind::Comma, Span { start: 19, length: 1 }),

            Token::new(TokenKind::Identifier("argv".into()), Span { start: 20, length: 4 }),
            Token::new(TokenKind::Colon, Span { start: 24, length: 1 }),
            Token::new(TokenKind::Identifier("todo".into()), Span { start: 25, length: 3 }),

            Token::new(TokenKind::CloseParen, Span { start: 28, length: 1 }),
            Token::new(TokenKind::OpenBrace, Span { start: 29, length: 1 }),
            Token::new(TokenKind::CloseBrace, Span { start: 30, length: 1 }),
        ]), @r#"
        Ok(
            [
                Statement {
                    kind: FunctionDeclaration(
                        FunctionDeclaration {
                            name: "main",
                            body: [],
                            parameters: [
                                FunctionParameter {
                                    name: "argc",
                                    type_expr: Named(
                                        "i32",
                                    ),
                                    type: Unknown,
                                    is_named: false,
                                    span: Span {
                                        start: 11,
                                        length: 8,
                                    },
                                },
                                FunctionParameter {
                                    name: "argv",
                                    type_expr: Named(
                                        "todo",
                                    ),
                                    type: Unknown,
                                    is_named: false,
                                    span: Span {
                                        start: 20,
                                        length: 8,
                                    },
                                },
                            ],
                            return_type_expr: None,
                            return_type: Unknown,
                        },
                    ),
                    span: Span {
                        start: 0,
                        length: 31,
                    },
                },
            ],
        )
        "#);
    }

    #[test]
    fn parse_function_declaration_with_explicit_return_type() {
        insta::assert_debug_snapshot!(ASTParser::new_and_parse(vec![
            Token::new(TokenKind::Keyword(Keyword::Func), Span { start: 0, length: 4 }),
            Token::new(TokenKind::Identifier("main".into()), Span { start: 5, length: 4 }),
            Token::new(TokenKind::OpenParen, Span { start: 10, length: 1 }),
            Token::new(TokenKind::CloseParen, Span { start: 11, length: 1 }),
            Token::new(TokenKind::Hyphen, Span { start: 12, length: 1 }),
            Token::new(TokenKind::RightAngleBracket, Span { start: 13, length: 1 }),
            Token::new(TokenKind::Identifier("i32".into()), Span { start: 14, length: 3 }),
            Token::new(TokenKind::OpenBrace, Span { start: 17, length: 1 }),
            Token::new(TokenKind::CloseBrace, Span { start: 18, length: 1 }),
        ]), @r#"
        Ok(
            [
                Statement {
                    kind: FunctionDeclaration(
                        FunctionDeclaration {
                            name: "main",
                            body: [],
                            parameters: [],
                            return_type_expr: Some(
                                Named(
                                    "i32",
                                ),
                            ),
                            return_type: Unknown,
                        },
                    ),
                    span: Span {
                        start: 0,
                        length: 19,
                    },
                },
            ],
        )
        "#);
    }

    #[test]
    fn parse_function_declaration_with_return_statement() {
        insta::assert_debug_snapshot!(ASTParser::new_and_parse(vec![
            Token::new(TokenKind::Keyword(Keyword::Func), Span { start: 0, length: 4 }),
            Token::new(TokenKind::Identifier("main".into()), Span { start: 5, length: 4 }),
            Token::new(TokenKind::OpenParen, Span { start: 10, length: 1 }),
            Token::new(TokenKind::CloseParen, Span { start: 11, length: 1 }),
            Token::new(TokenKind::OpenBrace, Span { start: 12, length: 1 }),
            Token::new(TokenKind::Keyword(Keyword::Return), Span { start: 13, length: 4 }),
            Token::new(TokenKind::Number(1234.0), Span { start: 17, length: 4 }),
            Token::new(TokenKind::Semicolon, Span { start: 21, length: 1 }),
            Token::new(TokenKind::CloseBrace, Span { start: 22, length: 1 }),
        ]), @r#"
        Ok(
            [
                Statement {
                    kind: FunctionDeclaration(
                        FunctionDeclaration {
                            name: "main",
                            body: [
                                Statement {
                                    kind: Return(
                                        Return {
                                            value: Some(
                                                Expression {
                                                    kind: NumberLiteral(
                                                        1234.0,
                                                    ),
                                                    span: Span {
                                                        start: 17,
                                                        length: 4,
                                                    },
                                                },
                                            ),
                                        },
                                    ),
                                    span: Span {
                                        start: 13,
                                        length: 8,
                                    },
                                },
                            ],
                            parameters: [],
                            return_type_expr: None,
                            return_type: Unknown,
                        },
                    ),
                    span: Span {
                        start: 0,
                        length: 23,
                    },
                },
            ],
        )
        "#);
    }

    #[test]
    fn parse_function_declaration_with_return_function_call_no_args() {
        insta::assert_debug_snapshot!(ASTParser::new_and_parse(vec![
            Token::new(TokenKind::Keyword(Keyword::Func), Span { start: 0, length: 4 }),
            Token::new(TokenKind::Identifier("main".into()), Span { start: 5, length: 4 }),
            Token::new(TokenKind::OpenParen, Span { start: 10, length: 1 }),
            Token::new(TokenKind::CloseParen, Span { start: 11, length: 1 }),
            Token::new(TokenKind::OpenBrace, Span { start: 12, length: 1 }),
            Token::new(TokenKind::Keyword(Keyword::Return), Span { start: 13, length: 4 }),
            Token::new(TokenKind::Identifier("foo".into()), Span { start: 17, length: 3 }),
            Token::new(TokenKind::OpenParen, Span { start: 20, length: 1 }),
            Token::new(TokenKind::CloseParen, Span { start: 21, length: 1 }),
            Token::new(TokenKind::Semicolon, Span { start: 22, length: 1 }),
            Token::new(TokenKind::CloseBrace, Span { start: 23, length: 1 }),
        ]), @r#"
        Ok(
            [
                Statement {
                    kind: FunctionDeclaration(
                        FunctionDeclaration {
                            name: "main",
                            body: [
                                Statement {
                                    kind: Return(
                                        Return {
                                            value: Some(
                                                Expression {
                                                    kind: FunctionCall(
                                                        FunctionCall {
                                                            name: "foo",
                                                            arguments: [],
                                                        },
                                                    ),
                                                    span: Span {
                                                        start: 17,
                                                        length: 5,
                                                    },
                                                },
                                            ),
                                        },
                                    ),
                                    span: Span {
                                        start: 13,
                                        length: 9,
                                    },
                                },
                            ],
                            parameters: [],
                            return_type_expr: None,
                            return_type: Unknown,
                        },
                    ),
                    span: Span {
                        start: 0,
                        length: 24,
                    },
                },
            ],
        )
        "#);
    }

    #[test]
    fn parse_function_declaration_with_return_function_call_with_arg() {
        insta::assert_debug_snapshot!(ASTParser::new_and_parse(vec![
            Token::new(TokenKind::Keyword(Keyword::Func), Span { start: 0, length: 4 }),
            Token::new(TokenKind::Identifier("main".into()), Span { start: 5, length: 4 }),
            Token::new(TokenKind::OpenParen, Span { start: 10, length: 1 }),
            Token::new(TokenKind::CloseParen, Span { start: 11, length: 1 }),
            Token::new(TokenKind::OpenBrace, Span { start: 12, length: 1 }),
            Token::new(TokenKind::Keyword(Keyword::Return), Span { start: 13, length: 4 }),
            Token::new(TokenKind::Identifier("foo".into()), Span { start: 17, length: 3 }),
            Token::new(TokenKind::OpenParen, Span { start: 20, length: 1 }),
            Token::new(TokenKind::Identifier("bar".into()), Span { start: 21, length: 3 }),
            Token::new(TokenKind::Colon, Span { start: 24, length: 1 }),
            Token::new(TokenKind::Identifier("ident".into()), Span { start: 25, length: 5 }),
            Token::new(TokenKind::CloseParen, Span { start: 30, length: 1 }),
            Token::new(TokenKind::Semicolon, Span { start: 31, length: 1 }),
            Token::new(TokenKind::CloseBrace, Span { start: 32, length: 1 }),
        ]), @r#"
        Ok(
            [
                Statement {
                    kind: FunctionDeclaration(
                        FunctionDeclaration {
                            name: "main",
                            body: [
                                Statement {
                                    kind: Return(
                                        Return {
                                            value: Some(
                                                Expression {
                                                    kind: FunctionCall(
                                                        FunctionCall {
                                                            name: "foo",
                                                            arguments: [
                                                                FunctionCallArgument {
                                                                    name: Some(
                                                                        "bar",
                                                                    ),
                                                                    value: Expression {
                                                                        kind: IdentifierReference(
                                                                            "ident",
                                                                        ),
                                                                        span: Span {
                                                                            start: 25,
                                                                            length: 5,
                                                                        },
                                                                    },
                                                                    span: Span {
                                                                        start: 21,
                                                                        length: 9,
                                                                    },
                                                                },
                                                            ],
                                                        },
                                                    ),
                                                    span: Span {
                                                        start: 17,
                                                        length: 14,
                                                    },
                                                },
                                            ),
                                        },
                                    ),
                                    span: Span {
                                        start: 13,
                                        length: 18,
                                    },
                                },
                            ],
                            parameters: [],
                            return_type_expr: None,
                            return_type: Unknown,
                        },
                    ),
                    span: Span {
                        start: 0,
                        length: 33,
                    },
                },
            ],
        )
        "#);
    }

    #[test]
    fn parse_function_declaration_with_return_function_call_with_args() {
        insta::assert_debug_snapshot!(ASTParser::new_and_parse(vec![
            Token::new(TokenKind::Keyword(Keyword::Func), Span { start: 0, length: 4 }),
            Token::new(TokenKind::Identifier("main".into()), Span { start: 5, length: 4 }),
            Token::new(TokenKind::OpenParen, Span { start: 10, length: 1 }),
            Token::new(TokenKind::CloseParen, Span { start: 11, length: 1 }),
            Token::new(TokenKind::OpenBrace, Span { start: 12, length: 1 }),
            Token::new(TokenKind::Keyword(Keyword::Return), Span { start: 13, length: 4 }),
            Token::new(TokenKind::Identifier("foo".into()), Span { start: 17, length: 3 }),
            Token::new(TokenKind::OpenParen, Span { start: 20, length: 1 }),
            Token::new(TokenKind::Identifier("bar".into()), Span { start: 21, length: 3 }),
            Token::new(TokenKind::Colon, Span { start: 24, length: 1 }),
            Token::new(TokenKind::Identifier("ident_a".into()), Span { start: 25, length: 7 }),
            Token::new(TokenKind::Comma, Span { start: 32, length: 1 }),
            Token::new(TokenKind::Identifier("baz".into()), Span { start: 33, length: 3 }),
            Token::new(TokenKind::Colon, Span { start: 36, length: 1 }),
            Token::new(TokenKind::Identifier("ident_b".into()), Span { start: 37, length: 7 }),
            Token::new(TokenKind::CloseParen, Span { start: 44, length: 1 }),
            Token::new(TokenKind::Semicolon, Span { start: 45, length: 1 }),
            Token::new(TokenKind::CloseBrace, Span { start: 46, length: 1 }),
        ]), @r#"
        Ok(
            [
                Statement {
                    kind: FunctionDeclaration(
                        FunctionDeclaration {
                            name: "main",
                            body: [
                                Statement {
                                    kind: Return(
                                        Return {
                                            value: Some(
                                                Expression {
                                                    kind: FunctionCall(
                                                        FunctionCall {
                                                            name: "foo",
                                                            arguments: [
                                                                FunctionCallArgument {
                                                                    name: Some(
                                                                        "bar",
                                                                    ),
                                                                    value: Expression {
                                                                        kind: IdentifierReference(
                                                                            "ident_a",
                                                                        ),
                                                                        span: Span {
                                                                            start: 25,
                                                                            length: 7,
                                                                        },
                                                                    },
                                                                    span: Span {
                                                                        start: 21,
                                                                        length: 11,
                                                                    },
                                                                },
                                                                FunctionCallArgument {
                                                                    name: Some(
                                                                        "baz",
                                                                    ),
                                                                    value: Expression {
                                                                        kind: IdentifierReference(
                                                                            "ident_b",
                                                                        ),
                                                                        span: Span {
                                                                            start: 37,
                                                                            length: 7,
                                                                        },
                                                                    },
                                                                    span: Span {
                                                                        start: 33,
                                                                        length: 11,
                                                                    },
                                                                },
                                                            ],
                                                        },
                                                    ),
                                                    span: Span {
                                                        start: 17,
                                                        length: 28,
                                                    },
                                                },
                                            ),
                                        },
                                    ),
                                    span: Span {
                                        start: 13,
                                        length: 32,
                                    },
                                },
                            ],
                            parameters: [],
                            return_type_expr: None,
                            return_type: Unknown,
                        },
                    ),
                    span: Span {
                        start: 0,
                        length: 47,
                    },
                },
            ],
        )
        "#);
    }

    #[test]
    fn parse_function_declaration_with_return_nested_function_call() {
        insta::assert_debug_snapshot!(ASTParser::new_and_parse(vec![
            Token::new(TokenKind::Keyword(Keyword::Func), Span { start: 0, length: 4 }),
            Token::new(TokenKind::Identifier("main".into()), Span { start: 5, length: 4 }),
            Token::new(TokenKind::OpenParen, Span { start: 10, length: 1 }),
            Token::new(TokenKind::CloseParen, Span { start: 11, length: 1 }),
            Token::new(TokenKind::OpenBrace, Span { start: 12, length: 1 }),
            Token::new(TokenKind::Keyword(Keyword::Return), Span { start: 13, length: 4 }),
            Token::new(TokenKind::Identifier("foo".into()), Span { start: 17, length: 3 }),
            Token::new(TokenKind::OpenParen, Span { start: 20, length: 1 }),
            Token::new(TokenKind::Identifier("baz".into()), Span { start: 21, length: 3 }),
            Token::new(TokenKind::Colon, Span { start: 24, length: 1 }),
            Token::new(TokenKind::Identifier("bar".into()), Span { start: 25, length: 3 }),
            Token::new(TokenKind::OpenParen, Span { start: 28, length: 1 }),
            Token::new(TokenKind::CloseParen, Span { start: 29, length: 1 }),
            Token::new(TokenKind::CloseParen, Span { start: 30, length: 1 }),
            Token::new(TokenKind::Semicolon, Span { start: 31, length: 1 }),
            Token::new(TokenKind::CloseBrace, Span { start: 32, length: 1 }),
        ]), @r#"
        Ok(
            [
                Statement {
                    kind: FunctionDeclaration(
                        FunctionDeclaration {
                            name: "main",
                            body: [
                                Statement {
                                    kind: Return(
                                        Return {
                                            value: Some(
                                                Expression {
                                                    kind: FunctionCall(
                                                        FunctionCall {
                                                            name: "foo",
                                                            arguments: [
                                                                FunctionCallArgument {
                                                                    name: Some(
                                                                        "baz",
                                                                    ),
                                                                    value: Expression {
                                                                        kind: FunctionCall(
                                                                            FunctionCall {
                                                                                name: "bar",
                                                                                arguments: [],
                                                                            },
                                                                        ),
                                                                        span: Span {
                                                                            start: 25,
                                                                            length: 5,
                                                                        },
                                                                    },
                                                                    span: Span {
                                                                        start: 21,
                                                                        length: 9,
                                                                    },
                                                                },
                                                            ],
                                                        },
                                                    ),
                                                    span: Span {
                                                        start: 17,
                                                        length: 14,
                                                    },
                                                },
                                            ),
                                        },
                                    ),
                                    span: Span {
                                        start: 13,
                                        length: 18,
                                    },
                                },
                            ],
                            parameters: [],
                            return_type_expr: None,
                            return_type: Unknown,
                        },
                    ),
                    span: Span {
                        start: 0,
                        length: 33,
                    },
                },
            ],
        )
        "#);
    }

    #[test]
    fn parse_function_declaration_with_variable_declaration() {
        insta::assert_debug_snapshot!(ASTParser::new_and_parse(vec![
            Token::new(TokenKind::Keyword(Keyword::Func), Span { start: 0, length: 4 }),
            Token::new(TokenKind::Identifier("main".into()), Span { start: 5, length: 4 }),
            Token::new(TokenKind::OpenParen, Span { start: 10, length: 1 }),
            Token::new(TokenKind::CloseParen, Span { start: 11, length: 1 }),
            Token::new(TokenKind::OpenBrace, Span { start: 12, length: 1 }),
            Token::new(TokenKind::Identifier("i32".into()), Span { start: 13, length: 4 }),
            Token::new(TokenKind::Identifier("variable".into()), Span { start: 17, length: 8 }),
            Token::new(TokenKind::Equals, Span { start: 25, length: 1 }),
            Token::new(TokenKind::Number(4.5), Span { start: 26, length: 3 }),
            Token::new(TokenKind::Semicolon, Span { start: 29, length: 1 }),
            Token::new(TokenKind::CloseBrace, Span { start: 30, length: 1 }),
        ]), @r#"
        Ok(
            [
                Statement {
                    kind: FunctionDeclaration(
                        FunctionDeclaration {
                            name: "main",
                            body: [
                                Statement {
                                    kind: VariableDeclaration(
                                        VariableDeclaration {
                                            name: "variable",
                                            type_expr: Named(
                                                "i32",
                                            ),
                                            type: Unknown,
                                            value: Expression {
                                                kind: NumberLiteral(
                                                    4.5,
                                                ),
                                                span: Span {
                                                    start: 26,
                                                    length: 3,
                                                },
                                            },
                                        },
                                    ),
                                    span: Span {
                                        start: 13,
                                        length: 16,
                                    },
                                },
                            ],
                            parameters: [],
                            return_type_expr: None,
                            return_type: Unknown,
                        },
                    ),
                    span: Span {
                        start: 0,
                        length: 31,
                    },
                },
            ],
        )
        "#);
    }

    #[test]
    fn parse_function_declaration_with_variable_assignment() {
        insta::assert_debug_snapshot!(ASTParser::new_and_parse(vec![
            Token::new(TokenKind::Keyword(Keyword::Func), Span { start: 0, length: 4 }),
            Token::new(TokenKind::Identifier("main".into()), Span { start: 5, length: 4 }),
            Token::new(TokenKind::OpenParen, Span { start: 10, length: 1 }),
            Token::new(TokenKind::CloseParen, Span { start: 11, length: 1 }),
            Token::new(TokenKind::OpenBrace, Span { start: 12, length: 1 }),
            Token::new(TokenKind::Identifier("variable".into()), Span { start: 13, length: 8 }),
            Token::new(TokenKind::Equals, Span { start: 21, length: 1 }),
            Token::new(TokenKind::Number(4.5), Span { start: 22, length: 3 }),
            Token::new(TokenKind::Semicolon, Span { start: 25, length: 1 }),
            Token::new(TokenKind::CloseBrace, Span { start: 26, length: 1 }),
        ]), @r#"
        Ok(
            [
                Statement {
                    kind: FunctionDeclaration(
                        FunctionDeclaration {
                            name: "main",
                            body: [
                                Statement {
                                    kind: VariableAssignment(
                                        VariableAssignment {
                                            name: "variable",
                                            value: Expression {
                                                kind: NumberLiteral(
                                                    4.5,
                                                ),
                                                span: Span {
                                                    start: 22,
                                                    length: 3,
                                                },
                                            },
                                        },
                                    ),
                                    span: Span {
                                        start: 13,
                                        length: 12,
                                    },
                                },
                            ],
                            parameters: [],
                            return_type_expr: None,
                            return_type: Unknown,
                        },
                    ),
                    span: Span {
                        start: 0,
                        length: 27,
                    },
                },
            ],
        )
        "#);
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
