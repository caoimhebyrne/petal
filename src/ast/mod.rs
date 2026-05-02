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
            StatementKind,
            function_declaration::{
                DeclarationModifier,
                FunctionDeclaration,
            },
            r#if::If,
            import::Import,
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
    module_registry::ModuleId,
    typechecker::r#type::Type,
};

pub mod error;
pub mod expression;
pub mod statement;
pub mod type_expr;

/// The AST parser.
pub struct ASTParser {
    /// The ID of the module being parsed.
    module_id: ModuleId,

    /// The tokens to parse into an AST.
    tokens: Vec<Token>,

    /// The position that the parser is at within the [tokens].
    cursor: usize,
}

impl ASTParser {
    /// Creates a new [ASTParser].
    pub fn new(module_id: ModuleId, tokens: Vec<Token>) -> Self {
        ASTParser { module_id, tokens, cursor: 0 }
    }

    /// Creates a new [ASTParser] instance and parses all of the provided [tokens] into an AST.
    pub fn new_and_parse(module_id: ModuleId, tokens: Vec<Token>) -> Result<Vec<Statement>, ASTError> {
        let mut parser = ASTParser::new(module_id, tokens);
        parser.parse()
    }

    /// Attempts to parse the [tokens] within this [ASTParser] into an AST.
    pub fn parse(&mut self) -> Result<Vec<Statement>, ASTError> {
        let mut statements: Vec<Statement> = vec![];

        while let Some(token) = self.peek() {
            let statement: Statement = match token.kind {
                TokenKind::Keyword(Keyword::Public) | TokenKind::Keyword(Keyword::Func) => {
                    self.parse_function_declaration()?
                }

                TokenKind::Keyword(Keyword::Import) => self.parse_import()?,

                _ => return Err(ASTErrorKind::UnexpectedToken(token.kind.clone()).at(token.span)),
            };

            statements.push(statement);
        }

        Ok(statements)
    }

    /// Attempts to parse a statement at the [ASTParser]'s current position.
    fn parse_statement(&mut self) -> Result<Statement, ASTError> {
        let token = self.peek_expect_any()?;

        let (statement, requires_semicolon) = match token.kind {
            TokenKind::Keyword(Keyword::Return) => (self.parse_return()?, true),

            TokenKind::Keyword(Keyword::If) => (self.parse_if()?, false),

            TokenKind::Identifier(_) => (
                if self.peek_nth(1).map(|it| it.kind == TokenKind::Equals).unwrap_or_default() {
                    self.parse_variable_assignment()?
                } else if self.peek_nth(1).map(|it| it.kind == TokenKind::OpenParen).unwrap_or_default() {
                    let (function_call, span) = self.parse_function_call()?;
                    Statement::new(StatementKind::FunctionCall(function_call), span)
                } else {
                    self.parse_variable_declaration()?
                },
                true,
            ),

            TokenKind::At if self.peek_nth(2).map(|it| it.kind == TokenKind::Equals).unwrap_or_default() => {
                (self.parse_variable_assignment()?, true)
            }

            _ => return Err(ASTErrorKind::ExpectedStatement(token.kind.clone()).at(token.span)),
        };

        if requires_semicolon {
            self.expect(TokenKind::Semicolon)?;
        }

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
        let left = self.parse_equals_or_not_equals_expression()?;

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

    /// Attempts to parse an equals or not equals expression at the [ASTParser]'s current position.
    fn parse_equals_or_not_equals_expression(&mut self) -> Result<Expression, ASTError> {
        let left = self.parse_value()?;

        let expression = if self.peek_is(TokenKind::Equals)
            && self.peek_nth(1).map(|it| it.kind == TokenKind::Equals).unwrap_or_default()
        {
            self.expect(TokenKind::Equals)?;
            self.expect(TokenKind::Equals)?;

            let right = self.parse_expression()?;
            let span = Span::between(left.span, right.span);

            Expression::new(BinaryOperation::new(left, right, BinaryOperand::Equals).into(), span)
        } else if self.peek_is(TokenKind::ExclamationMark) {
            self.expect(TokenKind::ExclamationMark)?;
            self.expect(TokenKind::Equals)?;

            let right = self.parse_expression()?;
            let span = Span::between(left.span, right.span);

            Expression::new(BinaryOperation::new(left, right, BinaryOperand::NotEquals).into(), span)
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

            TokenKind::At => {
                // This is a dereference, we must parse another expression to see what is being dereferenced.
                let span = token.span;
                self.consume();

                let inner = self.parse_value()?;
                let span = Span::between(span, inner.span);
                Expression::new(ExpressionKind::Dereference(inner.into()), span)
            }

            TokenKind::Ampersand => {
                // This is a reference, we must parse another expression to see what is being passed as a reference.
                let span = token.span;
                self.consume();

                let inner = self.parse_value()?;
                let span = Span::between(span, inner.span);
                Expression::new(ExpressionKind::Reference(inner.into()), span)
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
        // If a public keyword is present, then we can add the modifier.
        let is_public = if self.peek_is(TokenKind::Keyword(Keyword::Public)) {
            self.expect(TokenKind::Keyword(Keyword::Public))?;
            true
        } else {
            false
        };

        // All functions must start with the func keyword.
        let func_keyword_span = self.expect_span(TokenKind::Keyword(Keyword::Func))?;

        // Then, the name of the function must be present.
        let (function_name, _) = self.expect_identifier()?;

        let mut builder = FunctionDeclaration::builder(function_name);

        if is_public {
            builder = builder.modifier(DeclarationModifier::Public);
        }

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
            let (parameter_type, parameter_type_span) = self.parse_type_expr()?;

            builder = builder.parameter(
                parameter_name,
                parameter_type,
                Type::default(),
                is_named,
                Span::between(parameter_name_span, parameter_type_span),
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

            let (return_type, _) = self.parse_type_expr()?;

            builder = builder.return_type(return_type, Type::Unknown);
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
        let (type_expr, type_span) = self.parse_type_expr()?;

        // The next token must be the name of the variable.
        let (name, _) = self.expect_identifier()?;

        // The next token must be an equals.
        self.expect(TokenKind::Equals)?;

        // And finally, there must be an expression.
        let value = self.parse_expression()?;

        let span = Span::between(type_span, value.span);
        Ok(Statement::from(VariableDeclaration::new(name, type_expr, Type::Unknown, value), span))
    }

    /// Attempts to parse a variable assignment statement from the [ASTParser]'s current position.
    fn parse_variable_assignment(&mut self) -> Result<Statement, ASTError> {
        // The first token must be the name of the variable.
        let target = self.parse_expression()?;

        // The next token must be an equals.
        self.expect(TokenKind::Equals)?;

        // And finally, there must be an expression.
        let value = self.parse_expression()?;

        let span = Span::between(target.span, value.span);
        Ok(Statement::from(VariableAssignment::new(target, value), span))
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

    /// Attempts to parse an if statement from the [ASTParser]'s current position.
    fn parse_if(&mut self) -> Result<Statement, ASTError> {
        // The first token must be the if keyword.
        let if_keyword_span = self.expect_span(TokenKind::Keyword(Keyword::If))?;

        // Then there must be a condition.
        let condition = self.parse_expression()?;

        // And then the block of code to execute when the condition is true.
        let mut block: Vec<Statement> = Vec::new();
        self.expect(TokenKind::OpenBrace)?;

        while !self.peek_is(TokenKind::CloseBrace) {
            block.push(self.parse_statement()?);
        }

        let closing_brace_span = self.expect_span(TokenKind::CloseBrace)?;
        Ok(Statement::from(If::new(condition, block), Span::between(if_keyword_span, closing_brace_span)))
    }

    /// Attempts to parse an import statement from the [ASTParser]'s current position.
    fn parse_import(&mut self) -> Result<Statement, ASTError> {
        // The first token must be the import keyword.
        let import_keyword_span = self.expect_span(TokenKind::Keyword(Keyword::Import))?;

        // Then, there must be the name of the module to import.
        let (name, _) = self.expect_identifier()?;

        // And finally, there must be a semicolon.
        let semicolon_span = self.expect_span(TokenKind::Semicolon)?;

        Ok(Statement::from(Import::new(name), Span::between(import_keyword_span, semicolon_span)))
    }

    /// Attempts to parse a [`TypeExpr`] from the [`ASTParser`]'s current position.
    fn parse_type_expr(&mut self) -> Result<(TypeExpr, Span), ASTError> {
        // If the first token is an ampersand, then this is a reference type.
        if self.peek_is(TokenKind::Ampersand) {
            let ampersand_span = self.expect_span(TokenKind::Ampersand)?;

            let (inner, inner_span) = self.parse_type_expr()?;
            return Ok((TypeExpr::reference(inner), Span::between(ampersand_span, inner_span)));
        }

        // Otherwise, we can attempt to parse a named type.
        let (name, name_span) = self.expect_identifier()?;
        Ok((TypeExpr::named(name), name_span))
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
        let last_token_span = self.tokens.last().map(|it| it.span).unwrap_or(Span::new(self.module_id, 0, 0));
        self.consume().ok_or(ASTErrorKind::UnexpectedEndOfFile.at(last_token_span))
    }

    /// Expects a token to be at the [ASTParser]'s current position, advancing the cursor.
    /// An [ASTErrorKind::UnexpectedEndOfFile] will be returned if there are no tokens left in the stream.
    fn peek_expect_any(&self) -> Result<&Token, ASTError> {
        let last_token_span = self.tokens.last().map(|it| it.span).unwrap_or(Span::new(self.module_id, 0, 0));
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
        module_registry::MOCK_MODULE_ID,
    };

    fn assert_ast_error(tokens: Vec<Token>, error: ASTError) {
        assert_eq!(ASTParser::new_and_parse(MOCK_MODULE_ID, tokens), Err(error))
    }

    #[test]
    fn parse_function_declaration() {
        insta::assert_debug_snapshot!(ASTParser::new_and_parse(MOCK_MODULE_ID, vec![
            Token::new(TokenKind::Keyword(Keyword::Func), Span::new(MOCK_MODULE_ID, 0, 4)),
            Token::new(TokenKind::Identifier("main".into()), Span::new(MOCK_MODULE_ID, 5, 4)),
            Token::new(TokenKind::OpenParen, Span::new(MOCK_MODULE_ID, 10, 1)),
            Token::new(TokenKind::CloseParen, Span::new(MOCK_MODULE_ID, 11, 1)),
            Token::new(TokenKind::OpenBrace, Span::new(MOCK_MODULE_ID, 12, 1)),
                Token::new(TokenKind::CloseBrace, Span::new(MOCK_MODULE_ID, 13, 1)),
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
                            modifiers: [],
                        },
                    ),
                    span: Span {
                        module_id: ModuleId(
                            0,
                        ),
                        location: SpanLocation {
                            start: 0,
                            length: 14,
                        },
                    },
                },
            ],
        )
        "#);
    }

    #[test]
    fn parse_function_declaration_with_parameter() {
        insta::assert_debug_snapshot!(ASTParser::new_and_parse(MOCK_MODULE_ID, vec![
            Token::new(TokenKind::Keyword(Keyword::Func), Span::new(MOCK_MODULE_ID, 0, 4 )),
            Token::new(TokenKind::Identifier("main".into()), Span::new(MOCK_MODULE_ID, 5, 4 )),
            Token::new(TokenKind::OpenParen, Span::new(MOCK_MODULE_ID, 10, 1 )),
            Token::new(TokenKind::Identifier("argc".into()), Span::new(MOCK_MODULE_ID, 11, 4 )),
            Token::new(TokenKind::Colon, Span::new(MOCK_MODULE_ID, 15, 1 )),
            Token::new(TokenKind::Identifier("i32".into()), Span::new(MOCK_MODULE_ID, 16, 3 )),
            Token::new(TokenKind::CloseParen, Span::new(MOCK_MODULE_ID, 19, 1 )),
            Token::new(TokenKind::OpenBrace, Span::new(MOCK_MODULE_ID, 20, 1 )),
                Token::new(TokenKind::CloseBrace, Span::new(MOCK_MODULE_ID, 21, 1 )),
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
                                        module_id: ModuleId(
                                            0,
                                        ),
                                        location: SpanLocation {
                                            start: 11,
                                            length: 8,
                                        },
                                    },
                                },
                            ],
                            return_type_expr: None,
                            return_type: Unknown,
                            modifiers: [],
                        },
                    ),
                    span: Span {
                        module_id: ModuleId(
                            0,
                        ),
                        location: SpanLocation {
                            start: 0,
                            length: 22,
                        },
                    },
                },
            ],
        )
        "#);
    }

    #[test]
    fn parse_function_declaration_with_parameters() {
        insta::assert_debug_snapshot!(ASTParser::new_and_parse(MOCK_MODULE_ID, vec![
            Token::new(TokenKind::Keyword(Keyword::Func), Span::new(MOCK_MODULE_ID, 0, 4)),
            Token::new(TokenKind::Identifier("main".into()), Span::new(MOCK_MODULE_ID, 5, 4)),
            Token::new(TokenKind::OpenParen, Span::new(MOCK_MODULE_ID, 10, 1)),

            Token::new(TokenKind::Identifier("argc".into()), Span::new(MOCK_MODULE_ID, 11, 4)),
            Token::new(TokenKind::Colon, Span::new(MOCK_MODULE_ID, 15, 1)),
            Token::new(TokenKind::Identifier("i32".into()), Span::new(MOCK_MODULE_ID, 16, 3)),

            Token::new(TokenKind::Comma, Span::new(MOCK_MODULE_ID, 19, 1)),

            Token::new(TokenKind::Identifier("argv".into()), Span::new(MOCK_MODULE_ID, 20, 4)),
            Token::new(TokenKind::Colon, Span::new(MOCK_MODULE_ID, 24, 1)),
            Token::new(TokenKind::Identifier("todo".into()), Span::new(MOCK_MODULE_ID, 25, 3)),

            Token::new(TokenKind::CloseParen, Span::new(MOCK_MODULE_ID, 28, 1)),
            Token::new(TokenKind::OpenBrace, Span::new(MOCK_MODULE_ID, 29, 1)),
                Token::new(TokenKind::CloseBrace, Span::new(MOCK_MODULE_ID, 30, 1)),
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
                                        module_id: ModuleId(
                                            0,
                                        ),
                                        location: SpanLocation {
                                            start: 11,
                                            length: 8,
                                        },
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
                                        module_id: ModuleId(
                                            0,
                                        ),
                                        location: SpanLocation {
                                            start: 20,
                                            length: 8,
                                        },
                                    },
                                },
                            ],
                            return_type_expr: None,
                            return_type: Unknown,
                            modifiers: [],
                        },
                    ),
                    span: Span {
                        module_id: ModuleId(
                            0,
                        ),
                        location: SpanLocation {
                            start: 0,
                            length: 31,
                        },
                    },
                },
            ],
        )
        "#);
    }

    #[test]
    fn parse_function_declaration_with_explicit_return_type() {
        insta::assert_debug_snapshot!(ASTParser::new_and_parse(MOCK_MODULE_ID, vec![
            Token::new(TokenKind::Keyword(Keyword::Func), Span::new(MOCK_MODULE_ID, 0, 4)),
            Token::new(TokenKind::Identifier("main".into()), Span::new(MOCK_MODULE_ID, 5, 4)),
            Token::new(TokenKind::OpenParen, Span::new(MOCK_MODULE_ID, 10, 1)),
            Token::new(TokenKind::CloseParen, Span::new(MOCK_MODULE_ID, 11, 1)),
            Token::new(TokenKind::Hyphen, Span::new(MOCK_MODULE_ID, 12, 1)),
            Token::new(TokenKind::RightAngleBracket, Span::new(MOCK_MODULE_ID, 13, 1)),
            Token::new(TokenKind::Identifier("i32".into()), Span::new(MOCK_MODULE_ID, 14, 3)),
            Token::new(TokenKind::OpenBrace, Span::new(MOCK_MODULE_ID, 17, 1)),
                Token::new(TokenKind::CloseBrace, Span::new(MOCK_MODULE_ID, 18, 1)),
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
                            modifiers: [],
                        },
                    ),
                    span: Span {
                        module_id: ModuleId(
                            0,
                        ),
                        location: SpanLocation {
                            start: 0,
                            length: 19,
                        },
                    },
                },
            ],
        )
        "#);
    }

    #[test]
    fn parse_function_declaration_with_return_statement() {
        insta::assert_debug_snapshot!(ASTParser::new_and_parse(MOCK_MODULE_ID, vec![
            Token::new(TokenKind::Keyword(Keyword::Func), Span::new(MOCK_MODULE_ID, 0, 4)),
            Token::new(TokenKind::Identifier("main".into()), Span::new(MOCK_MODULE_ID, 5, 4)),
            Token::new(TokenKind::OpenParen, Span::new(MOCK_MODULE_ID, 10, 1)),
            Token::new(TokenKind::CloseParen, Span::new(MOCK_MODULE_ID, 11, 1)),
            Token::new(TokenKind::OpenBrace, Span::new(MOCK_MODULE_ID, 12, 1)),
            Token::new(TokenKind::Keyword(Keyword::Return), Span::new(MOCK_MODULE_ID, 13, 4)),
            Token::new(TokenKind::Number(1234.0), Span::new(MOCK_MODULE_ID, 17, 4)),
            Token::new(TokenKind::Semicolon, Span::new(MOCK_MODULE_ID, 21, 1)),
                Token::new(TokenKind::CloseBrace, Span::new(MOCK_MODULE_ID, 22, 1)),
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
                                                        module_id: ModuleId(
                                                            0,
                                                        ),
                                                        location: SpanLocation {
                                                            start: 17,
                                                            length: 4,
                                                        },
                                                    },
                                                },
                                            ),
                                        },
                                    ),
                                    span: Span {
                                        module_id: ModuleId(
                                            0,
                                        ),
                                        location: SpanLocation {
                                            start: 13,
                                            length: 8,
                                        },
                                    },
                                },
                            ],
                            parameters: [],
                            return_type_expr: None,
                            return_type: Unknown,
                            modifiers: [],
                        },
                    ),
                    span: Span {
                        module_id: ModuleId(
                            0,
                        ),
                        location: SpanLocation {
                            start: 0,
                            length: 23,
                        },
                    },
                },
            ],
        )
        "#);
    }

    #[test]
    fn parse_function_declaration_with_return_function_call_no_args() {
        insta::assert_debug_snapshot!(ASTParser::new_and_parse(MOCK_MODULE_ID, vec![
            Token::new(TokenKind::Keyword(Keyword::Func), Span::new(MOCK_MODULE_ID, 0, 4)),
            Token::new(TokenKind::Identifier("main".into()), Span::new(MOCK_MODULE_ID, 5, 4)),
            Token::new(TokenKind::OpenParen, Span::new(MOCK_MODULE_ID, 10, 1)),
            Token::new(TokenKind::CloseParen, Span::new(MOCK_MODULE_ID, 11, 1)),
            Token::new(TokenKind::OpenBrace, Span::new(MOCK_MODULE_ID, 12, 1)),
            Token::new(TokenKind::Keyword(Keyword::Return), Span::new(MOCK_MODULE_ID, 13, 4)),
            Token::new(TokenKind::Identifier("foo".into()), Span::new(MOCK_MODULE_ID, 17, 3)),
            Token::new(TokenKind::OpenParen, Span::new(MOCK_MODULE_ID, 20, 1)),
            Token::new(TokenKind::CloseParen, Span::new(MOCK_MODULE_ID, 21, 1)),
            Token::new(TokenKind::Semicolon, Span::new(MOCK_MODULE_ID, 22, 1)),
                Token::new(TokenKind::CloseBrace, Span::new(MOCK_MODULE_ID, 23, 1)),
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
                                                        module_id: ModuleId(
                                                            0,
                                                        ),
                                                        location: SpanLocation {
                                                            start: 17,
                                                            length: 5,
                                                        },
                                                    },
                                                },
                                            ),
                                        },
                                    ),
                                    span: Span {
                                        module_id: ModuleId(
                                            0,
                                        ),
                                        location: SpanLocation {
                                            start: 13,
                                            length: 9,
                                        },
                                    },
                                },
                            ],
                            parameters: [],
                            return_type_expr: None,
                            return_type: Unknown,
                            modifiers: [],
                        },
                    ),
                    span: Span {
                        module_id: ModuleId(
                            0,
                        ),
                        location: SpanLocation {
                            start: 0,
                            length: 24,
                        },
                    },
                },
            ],
        )
        "#);
    }

    #[test]
    fn parse_function_declaration_with_return_function_call_with_arg() {
        insta::assert_debug_snapshot!(ASTParser::new_and_parse(MOCK_MODULE_ID, vec![
            Token::new(TokenKind::Keyword(Keyword::Func), Span::new(MOCK_MODULE_ID, 0, 4)),
            Token::new(TokenKind::Identifier("main".into()), Span::new(MOCK_MODULE_ID, 5, 4)),
            Token::new(TokenKind::OpenParen, Span::new(MOCK_MODULE_ID, 10, 1)),
            Token::new(TokenKind::CloseParen, Span::new(MOCK_MODULE_ID, 11, 1)),
            Token::new(TokenKind::OpenBrace, Span::new(MOCK_MODULE_ID, 12, 1)),
            Token::new(TokenKind::Keyword(Keyword::Return), Span::new(MOCK_MODULE_ID, 13, 4)),
            Token::new(TokenKind::Identifier("foo".into()), Span::new(MOCK_MODULE_ID, 17, 3)),
            Token::new(TokenKind::OpenParen, Span::new(MOCK_MODULE_ID, 20, 1)),
            Token::new(TokenKind::Identifier("bar".into()), Span::new(MOCK_MODULE_ID, 21, 3)),
            Token::new(TokenKind::Colon, Span::new(MOCK_MODULE_ID, 24, 1)),
            Token::new(TokenKind::Identifier("ident".into()), Span::new(MOCK_MODULE_ID, 25, 5)),
            Token::new(TokenKind::CloseParen, Span::new(MOCK_MODULE_ID, 30, 1)),
            Token::new(TokenKind::Semicolon, Span::new(MOCK_MODULE_ID, 31, 1)),
                Token::new(TokenKind::CloseBrace, Span::new(MOCK_MODULE_ID, 32, 1)),
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
                                                                            module_id: ModuleId(
                                                                                0,
                                                                            ),
                                                                            location: SpanLocation {
                                                                                start: 25,
                                                                                length: 5,
                                                                            },
                                                                        },
                                                                    },
                                                                    span: Span {
                                                                        module_id: ModuleId(
                                                                            0,
                                                                        ),
                                                                        location: SpanLocation {
                                                                            start: 21,
                                                                            length: 9,
                                                                        },
                                                                    },
                                                                },
                                                            ],
                                                        },
                                                    ),
                                                    span: Span {
                                                        module_id: ModuleId(
                                                            0,
                                                        ),
                                                        location: SpanLocation {
                                                            start: 17,
                                                            length: 14,
                                                        },
                                                    },
                                                },
                                            ),
                                        },
                                    ),
                                    span: Span {
                                        module_id: ModuleId(
                                            0,
                                        ),
                                        location: SpanLocation {
                                            start: 13,
                                            length: 18,
                                        },
                                    },
                                },
                            ],
                            parameters: [],
                            return_type_expr: None,
                            return_type: Unknown,
                            modifiers: [],
                        },
                    ),
                    span: Span {
                        module_id: ModuleId(
                            0,
                        ),
                        location: SpanLocation {
                            start: 0,
                            length: 33,
                        },
                    },
                },
            ],
        )
        "#);
    }

    #[test]
    fn parse_function_declaration_with_return_function_call_with_args() {
        insta::assert_debug_snapshot!(ASTParser::new_and_parse(MOCK_MODULE_ID, vec![
            Token::new(TokenKind::Keyword(Keyword::Func), Span::new(MOCK_MODULE_ID, 0, 4 )),
            Token::new(TokenKind::Identifier("main".into()), Span::new(MOCK_MODULE_ID, 5, 4 )),
            Token::new(TokenKind::OpenParen, Span::new(MOCK_MODULE_ID, 10, 1 )),
            Token::new(TokenKind::CloseParen, Span::new(MOCK_MODULE_ID, 11, 1 )),
            Token::new(TokenKind::OpenBrace, Span::new(MOCK_MODULE_ID, 12, 1 )),
            Token::new(TokenKind::Keyword(Keyword::Return), Span::new(MOCK_MODULE_ID, 13, 4 )),
            Token::new(TokenKind::Identifier("foo".into()), Span::new(MOCK_MODULE_ID, 17, 3 )),
            Token::new(TokenKind::OpenParen, Span::new(MOCK_MODULE_ID, 20, 1 )),
            Token::new(TokenKind::Identifier("bar".into()), Span::new(MOCK_MODULE_ID, 21, 3 )),
            Token::new(TokenKind::Colon, Span::new(MOCK_MODULE_ID, 24, 1 )),
            Token::new(TokenKind::Identifier("ident_a".into()), Span::new(MOCK_MODULE_ID, 25, 7 )),
            Token::new(TokenKind::Comma, Span::new(MOCK_MODULE_ID, 32, 1 )),
            Token::new(TokenKind::Identifier("baz".into()), Span::new(MOCK_MODULE_ID, 33, 3 )),
            Token::new(TokenKind::Colon, Span::new(MOCK_MODULE_ID, 36, 1 )),
            Token::new(TokenKind::Identifier("ident_b".into()), Span::new(MOCK_MODULE_ID, 37, 7 )),
            Token::new(TokenKind::CloseParen, Span::new(MOCK_MODULE_ID, 44, 1 )),
            Token::new(TokenKind::Semicolon, Span::new(MOCK_MODULE_ID, 45, 1 )),
                Token::new(TokenKind::CloseBrace, Span::new(MOCK_MODULE_ID, 46, 1 )),
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
                                                                            module_id: ModuleId(
                                                                                0,
                                                                            ),
                                                                            location: SpanLocation {
                                                                                start: 25,
                                                                                length: 7,
                                                                            },
                                                                        },
                                                                    },
                                                                    span: Span {
                                                                        module_id: ModuleId(
                                                                            0,
                                                                        ),
                                                                        location: SpanLocation {
                                                                            start: 21,
                                                                            length: 11,
                                                                        },
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
                                                                            module_id: ModuleId(
                                                                                0,
                                                                            ),
                                                                            location: SpanLocation {
                                                                                start: 37,
                                                                                length: 7,
                                                                            },
                                                                        },
                                                                    },
                                                                    span: Span {
                                                                        module_id: ModuleId(
                                                                            0,
                                                                        ),
                                                                        location: SpanLocation {
                                                                            start: 33,
                                                                            length: 11,
                                                                        },
                                                                    },
                                                                },
                                                            ],
                                                        },
                                                    ),
                                                    span: Span {
                                                        module_id: ModuleId(
                                                            0,
                                                        ),
                                                        location: SpanLocation {
                                                            start: 17,
                                                            length: 28,
                                                        },
                                                    },
                                                },
                                            ),
                                        },
                                    ),
                                    span: Span {
                                        module_id: ModuleId(
                                            0,
                                        ),
                                        location: SpanLocation {
                                            start: 13,
                                            length: 32,
                                        },
                                    },
                                },
                            ],
                            parameters: [],
                            return_type_expr: None,
                            return_type: Unknown,
                            modifiers: [],
                        },
                    ),
                    span: Span {
                        module_id: ModuleId(
                            0,
                        ),
                        location: SpanLocation {
                            start: 0,
                            length: 47,
                        },
                    },
                },
            ],
        )
        "#);
    }

    #[test]
    fn parse_function_declaration_with_return_nested_function_call() {
        insta::assert_debug_snapshot!(ASTParser::new_and_parse(MOCK_MODULE_ID, vec![
            Token::new(TokenKind::Keyword(Keyword::Func), Span::new(MOCK_MODULE_ID, 0, 4 )),
            Token::new(TokenKind::Identifier("main".into()), Span::new(MOCK_MODULE_ID, 5, 4 )),
            Token::new(TokenKind::OpenParen, Span::new(MOCK_MODULE_ID, 10, 1 )),
            Token::new(TokenKind::CloseParen, Span::new(MOCK_MODULE_ID, 11, 1 )),
            Token::new(TokenKind::OpenBrace, Span::new(MOCK_MODULE_ID, 12, 1 )),
            Token::new(TokenKind::Keyword(Keyword::Return), Span::new(MOCK_MODULE_ID, 13, 4 )),
            Token::new(TokenKind::Identifier("foo".into()), Span::new(MOCK_MODULE_ID, 17, 3 )),
            Token::new(TokenKind::OpenParen, Span::new(MOCK_MODULE_ID, 20, 1 )),
            Token::new(TokenKind::Identifier("baz".into()), Span::new(MOCK_MODULE_ID, 21, 3 )),
            Token::new(TokenKind::Colon, Span::new(MOCK_MODULE_ID, 24, 1 )),
            Token::new(TokenKind::Identifier("bar".into()), Span::new(MOCK_MODULE_ID, 25, 3 )),
            Token::new(TokenKind::OpenParen, Span::new(MOCK_MODULE_ID, 28, 1 )),
            Token::new(TokenKind::CloseParen, Span::new(MOCK_MODULE_ID, 29, 1 )),
            Token::new(TokenKind::CloseParen, Span::new(MOCK_MODULE_ID, 30, 1 )),
            Token::new(TokenKind::Semicolon, Span::new(MOCK_MODULE_ID, 31, 1 )),
                Token::new(TokenKind::CloseBrace, Span::new(MOCK_MODULE_ID, 32, 1 )),
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
                                                                            module_id: ModuleId(
                                                                                0,
                                                                            ),
                                                                            location: SpanLocation {
                                                                                start: 25,
                                                                                length: 5,
                                                                            },
                                                                        },
                                                                    },
                                                                    span: Span {
                                                                        module_id: ModuleId(
                                                                            0,
                                                                        ),
                                                                        location: SpanLocation {
                                                                            start: 21,
                                                                            length: 9,
                                                                        },
                                                                    },
                                                                },
                                                            ],
                                                        },
                                                    ),
                                                    span: Span {
                                                        module_id: ModuleId(
                                                            0,
                                                        ),
                                                        location: SpanLocation {
                                                            start: 17,
                                                            length: 14,
                                                        },
                                                    },
                                                },
                                            ),
                                        },
                                    ),
                                    span: Span {
                                        module_id: ModuleId(
                                            0,
                                        ),
                                        location: SpanLocation {
                                            start: 13,
                                            length: 18,
                                        },
                                    },
                                },
                            ],
                            parameters: [],
                            return_type_expr: None,
                            return_type: Unknown,
                            modifiers: [],
                        },
                    ),
                    span: Span {
                        module_id: ModuleId(
                            0,
                        ),
                        location: SpanLocation {
                            start: 0,
                            length: 33,
                        },
                    },
                },
            ],
        )
        "#);
    }

    #[test]
    fn parse_function_declaration_with_variable_declaration() {
        insta::assert_debug_snapshot!(ASTParser::new_and_parse(MOCK_MODULE_ID, vec![
            Token::new(TokenKind::Keyword(Keyword::Func), Span::new(MOCK_MODULE_ID, 0, 4 )),
            Token::new(TokenKind::Identifier("main".into()), Span::new(MOCK_MODULE_ID, 5, 4 )),
            Token::new(TokenKind::OpenParen, Span::new(MOCK_MODULE_ID, 10, 1 )),
            Token::new(TokenKind::CloseParen, Span::new(MOCK_MODULE_ID, 11, 1 )),
            Token::new(TokenKind::OpenBrace, Span::new(MOCK_MODULE_ID, 12, 1 )),
            Token::new(TokenKind::Identifier("i32".into()), Span::new(MOCK_MODULE_ID, 13, 4 )),
            Token::new(TokenKind::Identifier("variable".into()), Span::new(MOCK_MODULE_ID, 17, 8 )),
            Token::new(TokenKind::Equals, Span::new(MOCK_MODULE_ID, 25, 1 )),
            Token::new(TokenKind::Number(4.5), Span::new(MOCK_MODULE_ID, 26, 3 )),
            Token::new(TokenKind::Semicolon, Span::new(MOCK_MODULE_ID, 29, 1 )),
                Token::new(TokenKind::CloseBrace, Span::new(MOCK_MODULE_ID, 30, 1 )),
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
                                                    module_id: ModuleId(
                                                        0,
                                                    ),
                                                    location: SpanLocation {
                                                        start: 26,
                                                        length: 3,
                                                    },
                                                },
                                            },
                                        },
                                    ),
                                    span: Span {
                                        module_id: ModuleId(
                                            0,
                                        ),
                                        location: SpanLocation {
                                            start: 13,
                                            length: 16,
                                        },
                                    },
                                },
                            ],
                            parameters: [],
                            return_type_expr: None,
                            return_type: Unknown,
                            modifiers: [],
                        },
                    ),
                    span: Span {
                        module_id: ModuleId(
                            0,
                        ),
                        location: SpanLocation {
                            start: 0,
                            length: 31,
                        },
                    },
                },
            ],
        )
        "#);
    }

    #[test]
    fn parse_function_declaration_with_variable_assignment() {
        insta::assert_debug_snapshot!(ASTParser::new_and_parse(MOCK_MODULE_ID, vec![
            Token::new(TokenKind::Keyword(Keyword::Func), Span::new(MOCK_MODULE_ID, 0, 4 )),
            Token::new(TokenKind::Identifier("main".into()), Span::new(MOCK_MODULE_ID, 5, 4 )),
            Token::new(TokenKind::OpenParen, Span::new(MOCK_MODULE_ID, 10, 1 )),
            Token::new(TokenKind::CloseParen, Span::new(MOCK_MODULE_ID, 11, 1 )),
            Token::new(TokenKind::OpenBrace, Span::new(MOCK_MODULE_ID, 12, 1 )),
            Token::new(TokenKind::Identifier("variable".into()), Span::new(MOCK_MODULE_ID, 13, 8 )),
            Token::new(TokenKind::Equals, Span::new(MOCK_MODULE_ID, 21, 1 )),
            Token::new(TokenKind::Number(4.5), Span::new(MOCK_MODULE_ID, 22, 3 )),
            Token::new(TokenKind::Semicolon, Span::new(MOCK_MODULE_ID, 25, 1 )),
                Token::new(TokenKind::CloseBrace, Span::new(MOCK_MODULE_ID, 26, 1 )),
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
                                            target: Expression {
                                                kind: IdentifierReference(
                                                    "variable",
                                                ),
                                                span: Span {
                                                    module_id: ModuleId(
                                                        0,
                                                    ),
                                                    location: SpanLocation {
                                                        start: 13,
                                                        length: 8,
                                                    },
                                                },
                                            },
                                            value: Expression {
                                                kind: NumberLiteral(
                                                    4.5,
                                                ),
                                                span: Span {
                                                    module_id: ModuleId(
                                                        0,
                                                    ),
                                                    location: SpanLocation {
                                                        start: 22,
                                                        length: 3,
                                                    },
                                                },
                                            },
                                        },
                                    ),
                                    span: Span {
                                        module_id: ModuleId(
                                            0,
                                        ),
                                        location: SpanLocation {
                                            start: 13,
                                            length: 12,
                                        },
                                    },
                                },
                            ],
                            parameters: [],
                            return_type_expr: None,
                            return_type: Unknown,
                            modifiers: [],
                        },
                    ),
                    span: Span {
                        module_id: ModuleId(
                            0,
                        ),
                        location: SpanLocation {
                            start: 0,
                            length: 27,
                        },
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
                Token::new(TokenKind::Keyword(Keyword::Func), Span::new(MOCK_MODULE_ID, 0, 4)),
                Token::new(TokenKind::Identifier("main".into()), Span::new(MOCK_MODULE_ID, 5, 4)),
            ],
            ASTErrorKind::UnexpectedEndOfFile.at(Span::new(MOCK_MODULE_ID, 5, 4)),
        );
    }
}
