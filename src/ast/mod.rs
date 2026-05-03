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
                BinaryOperation,
                BinaryOperator,
            },
            function_call::FunctionCall,
            member_access::MemberAccess,
            structure_initialization::StructureInitialization,
        },
        statement::{
            Statement,
            function_declaration::{
                DeclarationModifier,
                FunctionDeclaration,
            },
            r#if::If,
            import::Import,
            r#return::Return,
            type_declaration::TypeDeclaration,
            variable_assignment::VariableAssignment,
            variable_declaration::VariableDeclaration,
        },
        type_expr::{
            StructureField,
            TypeExpr,
        },
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

                TokenKind::Keyword(Keyword::Type) => self.parse_type_declaration()?,

                TokenKind::Keyword(Keyword::Import) => self.parse_import()?,

                _ => return Err(ASTErrorKind::UnexpectedToken(token.kind.clone()).at(token.span)),
            };

            statements.push(statement);
        }

        Ok(statements)
    }

    /// Attempts to parse a statement at the [ASTParser]'s current position.
    fn parse_statement(&mut self) -> Result<Statement, ASTError> {
        let token = self.peek_expect_any().cloned()?;

        let (statement, requires_semicolon) = match &token.kind {
            TokenKind::Keyword(Keyword::Return) => (self.parse_return()?, true),

            TokenKind::Keyword(Keyword::If) => (self.parse_if()?, false),

            TokenKind::Identifier(_) if self.peek_nth(1).map(|it| it.kind == TokenKind::Colon).unwrap_or_default() => {
                (self.parse_variable_declaration()?, true)
            }

            TokenKind::At | TokenKind::Identifier(_) => {
                let expression = self.parse_expression()?;

                if self.peek_is(TokenKind::Equals) {
                    self.expect(TokenKind::Equals)?;
                    let value = self.parse_expression()?;

                    let span = Span::between(expression.span, value.span);
                    (Statement::from(VariableAssignment::new(expression, value), span), true)
                } else if let ExpressionKind::FunctionCall(function_call) = expression.kind {
                    (Statement::from(function_call, expression.span), true)
                } else {
                    return Err(ASTErrorKind::ExpectedStatement(token.kind.clone()).at(token.span));
                }
            }

            _ => return Err(ASTErrorKind::ExpectedStatement(token.kind.clone()).at(token.span)),
        };

        if requires_semicolon {
            self.expect(TokenKind::Semicolon)?;
        }

        Ok(statement)
    }

    fn parse_expression(&mut self) -> Result<Expression, ASTError> {
        self.parse_expression_with_precedence(0)
    }

    /// Attempts to parse an expression at the [ASTParser]'s current position.
    fn parse_expression_with_precedence(&mut self, precedence: u8) -> Result<Expression, ASTError> {
        let mut left = if self.peek_is(TokenKind::OpenParen) {
            self.expect(TokenKind::OpenParen)?;
            let expression = self.parse_expression_with_precedence(0)?;
            self.expect(TokenKind::CloseParen)?;

            expression
        } else {
            self.parse_value()?
        };

        while let Some(operator) = self.peek_and_parse_binary_operator() {
            if operator.precedence() < precedence {
                break;
            }

            self.consume();

            let right = self.parse_expression_with_precedence(operator.precedence() + 1)?;
            let span = Span::between(left.span, right.span);

            left = Expression::new(BinaryOperation::new(left, right, operator).into(), span);
        }

        Ok(left)
    }

    fn peek_and_parse_binary_operator(&mut self) -> Option<BinaryOperator> {
        let operator = match self.peek()?.kind {
            TokenKind::Plus => BinaryOperator::Add,
            TokenKind::Hyphen => BinaryOperator::Subtract,
            TokenKind::Asterisk => BinaryOperator::Multiply,
            TokenKind::ForwardSlash => BinaryOperator::Divide,
            TokenKind::DoubleEquals => BinaryOperator::Equals,
            TokenKind::NotEquals => BinaryOperator::NotEquals,

            _ => return None,
        };

        Some(operator)
    }

    /// Attempts to parse a simple value at the [ASTParser]'s current position.
    fn parse_value(&mut self) -> Result<Expression, ASTError> {
        let token = self.peek_expect_any()?;

        // FIXME: We need to copy the span before attempting to acquire a mutable reference via consume.
        let span = token.span;

        let mut expression = match &token.kind {
            TokenKind::Number(value) => {
                let value = *value;
                self.consume();
                Expression::new(ExpressionKind::NumberLiteral(value), span)
            }

            TokenKind::Keyword(Keyword::True) => {
                self.consume();
                Expression::new(ExpressionKind::BooleanLiteral(true), span)
            }

            TokenKind::Keyword(Keyword::False) => {
                self.consume();
                Expression::new(ExpressionKind::BooleanLiteral(false), span)
            }

            TokenKind::At => {
                self.consume();

                // This is a dereference, we must parse another expression to see what is being dereferenced.
                let inner = self.parse_value()?;
                let span = Span::between(span, inner.span);
                Expression::new(ExpressionKind::Dereference(inner.into()), span)
            }

            TokenKind::Ampersand => {
                self.consume();

                // This is a reference, we must parse another expression to see what is being passed as a reference.
                let inner = self.parse_value()?;
                let span = Span::between(span, inner.span);
                Expression::new(ExpressionKind::Reference(inner.into()), span)
            }

            TokenKind::Identifier(name) => {
                let name = name.clone();

                self.consume();
                Expression::new(ExpressionKind::IdentifierReference(name), span)
            }

            TokenKind::OpenBrace => self.parse_structure_initialization()?,

            _ => return Err(ASTErrorKind::ExpectedExpression(token.kind.clone()).at(token.span)),
        };

        loop {
            let should_dereference_target = if self.peek_is(TokenKind::Period) {
                self.expect(TokenKind::Period)?;
                false
            } else if self.peek_is(TokenKind::Hyphen)
                && self.peek_nth(1).map(|it| it.kind == TokenKind::RightAngleBracket).unwrap_or_default()
            {
                self.expect(TokenKind::Hyphen)?;
                self.expect(TokenKind::RightAngleBracket)?;
                true
            } else {
                break;
            };

            if should_dereference_target {
                let span = expression.span;
                expression = Expression::new(ExpressionKind::Dereference(expression.into()), span);
            }

            let (member_name, member_span) = self.expect_identifier()?;
            let span = Span::between(expression.span, member_span);

            expression = Expression::new(MemberAccess::new(expression, member_name).into(), span);
        }

        // After all of that, we can attempt to parse a function call if it is present.
        if self.peek_is(TokenKind::OpenParen) {
            // The expression that we have collected up until this point is considered to be the callee of the function call.
            let (function_call, function_call_span) = self.parse_function_call(expression)?;
            expression = Expression::new(function_call.into(), function_call_span)
        }

        Ok(expression)
    }

    /// Attempts to parse a structure initialization expression.
    fn parse_structure_initialization(&mut self) -> Result<Expression, ASTError> {
        let mut builder = StructureInitialization::builder();

        // The first token must be an open brace.
        let open_brace_span = self.expect(TokenKind::OpenBrace)?.span;

        while !self.peek_is(TokenKind::CloseBrace) {
            let period_span = self.expect(TokenKind::Period)?.span;
            let (field_name, _) = self.expect_identifier()?;

            self.expect(TokenKind::Equals)?;

            let value = self.parse_expression()?;

            let span = Span::between(period_span, value.span);
            builder = builder.field(field_name, value, Span::between(period_span, span));

            if self.peek_is(TokenKind::CloseBrace) {
                break;
            }

            self.expect(TokenKind::Comma)?;
        }

        // The last token must be a closing brace.
        let close_brace_span = self.expect(TokenKind::CloseBrace)?.span;

        Ok(Expression::new(builder.build().into(), Span::between(open_brace_span, close_brace_span)))
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
        let func_keyword_span = self.expect(TokenKind::Keyword(Keyword::Func))?.span;

        // Then, the type name of the owner of the function might be specified.
        let owner_type_name = if self.peek_nth(1).map(|it| it.kind == TokenKind::Period).unwrap_or_default() {
            let (name, _) = self.expect_identifier()?;
            self.expect(TokenKind::Period)?;
            Some(name)
        } else {
            None
        };

        // Then, the name of the function must be present.
        let (function_name, _) = self.expect_identifier()?;

        let mut builder = FunctionDeclaration::builder(function_name);

        if is_public {
            builder = builder.modifier(DeclarationModifier::Public);
        }

        if let Some(name) = owner_type_name {
            builder = builder.owner_type_name(name);
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

        let closing_brace_span = self.expect(TokenKind::CloseBrace)?.span;

        Ok(Statement::new(builder.build().into(), Span::between(func_keyword_span, closing_brace_span)))
    }

    /// Attempts to parse a return statement from the [ASTParser]'s current position.
    fn parse_return(&mut self) -> Result<Statement, ASTError> {
        let return_keyword_span = self.expect(TokenKind::Keyword(Keyword::Return))?.span;

        if self.peek_is(TokenKind::Semicolon) {
            return Ok(Statement::from(Return::new(None), return_keyword_span));
        }

        let value = self.parse_expression()?;
        let span = Span::between(return_keyword_span, value.span);
        Ok(Statement::from(Return::new(Some(value)), span))
    }

    /// Attempts to parse a variable declaration statement from the [ASTParser]'s current position.
    fn parse_variable_declaration(&mut self) -> Result<Statement, ASTError> {
        // The first token must be the name of the variable.
        let (name, name_span) = self.expect_identifier()?;

        self.expect(TokenKind::Colon)?;

        // The next token must be the type of the variable.
        let (type_expr, type_span) = self.parse_type_expr()?;

        // And finally, there must be an expression.
        let (value, span) = if self.peek_is(TokenKind::Semicolon) {
            (None, Span::between(name_span, type_span))
        } else {
            // The next token must be an equals.
            self.expect(TokenKind::Equals)?;

            let value = self.parse_expression()?;
            let span = Span::between(type_span, value.span);

            (Some(value), span)
        };

        Ok(Statement::from(VariableDeclaration::new(name, type_expr, Type::Unknown, value), span))
    }

    /// Attempts to parse a function call from the [ASTParser]'s current position.
    fn parse_function_call(&mut self, function_callee: Expression) -> Result<(FunctionCall, Span), ASTError> {
        let function_callee_span = function_callee.span;

        let mut builder = FunctionCall::builder(function_callee);

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

        let close_paren_span = self.expect(TokenKind::CloseParen)?.span;

        Ok((builder.build(), Span::between(function_callee_span, close_paren_span)))
    }

    /// Attempts to parse an if statement from the [ASTParser]'s current position.
    fn parse_if(&mut self) -> Result<Statement, ASTError> {
        // The first token must be the if keyword.
        let if_keyword_span = self.expect(TokenKind::Keyword(Keyword::If))?.span;

        // Then there must be a condition.
        let condition = self.parse_expression()?;

        // And then the block of code to execute when the condition is true.
        let mut block: Vec<Statement> = Vec::new();
        self.expect(TokenKind::OpenBrace)?;

        while !self.peek_is(TokenKind::CloseBrace) {
            block.push(self.parse_statement()?);
        }

        let closing_brace_span = self.expect(TokenKind::CloseBrace)?.span;
        Ok(Statement::from(If::new(condition, block), Span::between(if_keyword_span, closing_brace_span)))
    }

    /// Attempts to parse an import statement from the [ASTParser]'s current position.
    fn parse_import(&mut self) -> Result<Statement, ASTError> {
        // The first token must be the import keyword.
        let import_keyword_span = self.expect(TokenKind::Keyword(Keyword::Import))?.span;

        // Then, there must be the name of the module to import.
        let (name, _) = self.expect_identifier()?;

        // And finally, there must be a semicolon.
        let semicolon_span = self.expect(TokenKind::Semicolon)?.span;

        Ok(Statement::from(Import::new(name), Span::between(import_keyword_span, semicolon_span)))
    }

    /// Attempts to parse a type declaration statement from the [ASTParser]'s current position.
    fn parse_type_declaration(&mut self) -> Result<Statement, ASTError> {
        // The first token must be the `type` keyword.
        let type_keyword_span = self.expect(TokenKind::Keyword(Keyword::Type))?.span;

        // Then, there must be the name of the type.
        let (name, _) = self.expect_identifier()?;

        // Then there must be an equals.
        self.expect(TokenKind::Equals)?;

        // And finally, an expression must be present for the type, followed by a semicolon.
        let (type_expr, _) = self.parse_type_expr()?;
        let semicolon_span = self.expect(TokenKind::Semicolon)?.span;

        Ok(Statement::from(TypeDeclaration::new(name, type_expr), Span::between(type_keyword_span, semicolon_span)))
    }

    /// Attempts to parse a [`TypeExpr`] from the [`ASTParser`]'s current position.
    fn parse_type_expr(&mut self) -> Result<(TypeExpr, Span), ASTError> {
        // If the first token is an ampersand, then this is a reference type.
        if self.peek_is(TokenKind::Ampersand) {
            let ampersand_span = self.expect(TokenKind::Ampersand)?.span;

            let (inner, inner_span) = self.parse_type_expr()?;
            return Ok((TypeExpr::reference(inner), Span::between(ampersand_span, inner_span)));
        }

        // If the first token is a question mark, then this is a optional type.
        if self.peek_is(TokenKind::QuestionMark) {
            let ampersand_span = self.expect(TokenKind::QuestionMark)?.span;

            let (inner, inner_span) = self.parse_type_expr()?;
            return Ok((TypeExpr::Optional(inner.into()), Span::between(ampersand_span, inner_span)));
        }

        // If the first token is the `struct` keyword, then we are parsing a structure definition.
        if self.peek_is(TokenKind::Keyword(Keyword::Struct)) {
            let struct_span = self.expect(TokenKind::Keyword(Keyword::Struct))?.span;

            self.expect(TokenKind::OpenBrace)?;

            let mut fields: Vec<StructureField> = Vec::new();

            while !self.peek_is(TokenKind::CloseBrace) {
                let (field_name, field_name_span) = self.expect_identifier()?;
                self.expect(TokenKind::Colon)?;
                let (field_type, field_type_span) = self.parse_type_expr()?;

                fields.push(StructureField::new(
                    field_name,
                    field_type,
                    Span::between(field_name_span, field_type_span),
                ));

                if self.peek_is(TokenKind::CloseBrace) {
                    break;
                }

                self.expect(TokenKind::Comma)?;
            }

            let close_brace_span = self.expect(TokenKind::CloseBrace)?.span;
            return Ok((TypeExpr::Structure { fields }, Span::between(struct_span, close_brace_span)));
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

    use crate::{
        ast::{
            ASTParser,
            expression::{
                Expression,
                ExpressionKind,
                binary_operation::{
                    BinaryOperation,
                    BinaryOperator,
                },
            },
        },
        core::span::Span,
        lexer::Lexer,
        module_registry::MOCK_MODULE_ID,
    };

    /// Runs the lexer on the provided source string, and attempts to parse a single expression from it.
    fn parse_expression_from_str(string: &str) -> Expression {
        let mut lexer = Lexer::new(MOCK_MODULE_ID, string);
        let mut parser = ASTParser::new(MOCK_MODULE_ID, lexer.parse().expect("Lexer failed to parse input string"));
        parser.parse_expression().expect("Parser failed to parse expression")
    }

    /// Creates an identifier reference expression from the provided string.
    fn ident(value: &str) -> Expression {
        Expression::new(ExpressionKind::IdentifierReference(value.into()), Span::new(MOCK_MODULE_ID, 0, 0))
    }

    /// Creates a binary operation expression using the provided operator and values.
    fn binop(left: Expression, operator: BinaryOperator, right: Expression) -> Expression {
        Expression::new(BinaryOperation::new(left, right, operator).into(), Span::new(MOCK_MODULE_ID, 0, 0))
    }

    /// Asserts that the provided [`Expression`]s are the same, ignoring any differences in their [`Span`]s.
    fn assert_expression_eq(mut a: Expression, mut b: Expression) {
        remove_spans(&mut a);
        remove_spans(&mut b);

        assert_eq!(a, b);
    }

    /// Substitues the [`Span`] in the provided [`Expression`] with a default value.
    fn remove_spans(expression: &mut Expression) {
        expression.span = Span::new(MOCK_MODULE_ID, 0, 0);

        match &mut expression.kind {
            ExpressionKind::BinaryOperation(binary_operation) => {
                remove_spans(&mut binary_operation.left);
                remove_spans(&mut binary_operation.right);
            }

            ExpressionKind::Dereference(dereference) => {
                remove_spans(dereference);
            }

            ExpressionKind::FunctionCall(function_call) => {
                for argument in &mut function_call.arguments {
                    remove_spans(&mut argument.value);
                    argument.span = Span::new(MOCK_MODULE_ID, 0, 0);
                }
            }

            ExpressionKind::Reference(reference) => {
                remove_spans(reference);
            }

            ExpressionKind::StructureInitialization(structure_initialization) => {
                for field in &mut structure_initialization.fields {
                    remove_spans(&mut field.value);
                    field.span = Span::new(MOCK_MODULE_ID, 0, 0);
                }
            }

            ExpressionKind::MemberAccess(member_access) => {
                remove_spans(&mut member_access.target);
            }

            ExpressionKind::OptionalWrap(optional_wrap) => {
                remove_spans(&mut optional_wrap.inner_value);
            }

            // These expressions do not have any children.
            ExpressionKind::BooleanLiteral(_) => {}
            ExpressionKind::IdentifierReference(_) => {}
            ExpressionKind::NumberLiteral(_) => {}
        }
    }

    #[test]
    fn operator_precedence_and_associativity() {
        // Arrange
        let cases = [
            // Subtraction must be associative to the left.
            (
                "a - b - c",
                binop(binop(ident("a"), BinaryOperator::Subtract, ident("b")), BinaryOperator::Subtract, ident("c")),
            ),
            // Parenthesis should force grouping of a binop.
            (
                "(a + b) * c",
                binop(binop(ident("a"), BinaryOperator::Add, ident("b")), BinaryOperator::Multiply, ident("c")),
            ),
            // Multiplication takes precedence over addition.
            (
                "a + b * c",
                binop(ident("a"), BinaryOperator::Add, binop(ident("b"), BinaryOperator::Multiply, ident("c"))),
            ),
        ];

        for (src, expected) in cases {
            println!("Validating operator precedence and associativity case '{src}'");
            assert_expression_eq(parse_expression_from_str(src), expected)
        }
    }
}
