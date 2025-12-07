use crate::{
    error::ASTError,
    expression::{
        ExpressionNode,
        binary_operation::{BinaryOperation, BinaryOperationKind},
        boolean_literal::BooleanLiteral,
        identifier_reference::IdentifierReference,
        integer_literal::IntegerLiteral,
        reference::Reference,
        string_literal::StringLiteral,
    },
    node::FunctionCall,
    statement::{
        StatementNode, TopLevelStatementNode,
        function_declaration::{FunctionDeclaration, FunctionModifier, FunctionParameter},
        r#if::If,
        import::Import,
        r#return::Return,
        variable_assignment::VariableAssignment,
        variable_declaration::VariableDeclaration,
        while_loop::WhileLoop,
    },
    token_stream_ext::TokenStreamExt,
};
use petal_core::{
    error::Result,
    source_span::SourceSpan,
    r#type::{ResolvedType, Type, TypeReference, pool::TypePool},
};
use petal_lexer::{
    stream::TokenStream,
    token::{Keyword, TokenKind},
};

/// Parses an abstract syntax tree for a Petal module by consuming tokens from a [TokenStream] and converting them
/// into [TopLevelStatementNode]s.
pub struct ASTParser<'ctx> {
    /// The [TokenStream] to consume from.
    stream: TokenStream,

    /// The [TypePool] to allocate types in.
    type_pool: &'ctx mut TypePool,
}

impl<'ctx> ASTParser<'ctx> {
    /// Instantiates a new [ASTParser].
    pub fn new(stream: TokenStream, type_pool: &'ctx mut TypePool) -> Self {
        ASTParser { stream, type_pool }
    }

    /// Parses the tokens within this [ASTParser] into a [Vec] of [TopLevelStatementNode]s.
    pub fn parse(&mut self) -> Result<Vec<TopLevelStatementNode>> {
        let mut statements = vec![];

        while self.stream.has_remaining() {
            statements.push(self.parse_top_level_statement()?);
        }

        Ok(statements)
    }

    /// Attempts to parse a top level statement from the [ASTParser]'s current position.
    fn parse_top_level_statement(&mut self) -> Result<TopLevelStatementNode> {
        self.stream.consume_all_whitespace();

        let token = self.stream.peek_or_err()?;

        let (node, expect_semicolon) = match token.kind {
            TokenKind::Keyword(Keyword::Func) | TokenKind::Keyword(Keyword::Extern) => (
                TopLevelStatementNode::from_pair(self.parse_function_declaration()?),
                false,
            ),

            TokenKind::Keyword(Keyword::Import) => (TopLevelStatementNode::from_pair(self.parse_import()?), true),

            _ => return ASTError::unexpected_token(*token).into(),
        };

        if expect_semicolon {
            self.stream.expect(TokenKind::Semicolon)?;
        }

        Ok(node)
    }

    /// Attempts to parse a statement from the [ASTParser]'s current position.
    fn parse_statement(&mut self) -> Result<StatementNode> {
        self.stream.consume_all_whitespace();

        let token = *self.stream.peek_or_err()?;

        let (node, expect_semicolon) = match token.kind {
            TokenKind::Keyword(Keyword::Return) => (StatementNode::from_pair(self.parse_return()?), true),

            // if <condition> { ... }
            TokenKind::Keyword(Keyword::If) => (StatementNode::from_pair(self.parse_if()?), false),

            // while <condition> { ... }
            TokenKind::Keyword(Keyword::While) => (StatementNode::from_pair(self.parse_while()?), false),

            // &<type> <identifier> = <expression>;
            TokenKind::Ampersand => (StatementNode::from_pair(self.parse_variable_declaration()?), true),

            // (<type>) <identifier> = <expression>;
            TokenKind::Identifier(_) if self.stream.nth_is(2, TokenKind::Equals) => {
                (StatementNode::from_pair(self.parse_variable_declaration()?), true)
            }

            // <identifier> = <expression>;
            TokenKind::Identifier(_) if self.stream.nth_is(1, TokenKind::Equals) => {
                (StatementNode::from_pair(self.parse_variable_assignment()?), true)
            }

            // <name>(...)
            TokenKind::Identifier(_) if self.stream.nth_is(1, TokenKind::LeftParenthesis) => {
                (StatementNode::from_pair(self.parse_function_call()?), true)
            }

            _ => return ASTError::unexpected_token(token).into(),
        };

        if expect_semicolon {
            self.stream.expect(TokenKind::Semicolon)?;
        }

        Ok(node)
    }

    /// Attempts to parse an expression from the [ASTParser]'s current position.
    fn parse_expression(&mut self) -> Result<ExpressionNode> {
        self.parse_add_or_sub_binary_operation()
    }

    /// Attempts to parse an addition or subtraction binary operation at the [ASTParser]'s current position.
    fn parse_add_or_sub_binary_operation(&mut self) -> Result<ExpressionNode> {
        let left = self.parse_mul_or_div_binary_operation()?;

        let operator_token = match self.stream.peek() {
            Some(token) => *token,
            None => return Ok(left),
        };

        let binary_operation_kind = match operator_token.kind {
            TokenKind::Plus => BinaryOperationKind::Add,
            TokenKind::Hyphen => BinaryOperationKind::Subtract,

            // The next token is not compatible with a binary operation.
            _ => return Ok(left),
        };

        // We can consume the operation token now.
        self.stream.consume_or_err()?;

        let right = self.parse_expression()?;
        let span = SourceSpan::between(&left.span, &right.span);

        Ok(ExpressionNode::from(
            BinaryOperation::new(binary_operation_kind, left, right),
            span,
        ))
    }

    /// Attempts to parse a multiplication or division binary operation at the [ASTParser]'s current position.
    fn parse_mul_or_div_binary_operation(&mut self) -> Result<ExpressionNode> {
        let left = self.parse_equals_binary_operation()?;

        let operator_token = match self.stream.peek() {
            Some(token) => *token,
            None => return Ok(left),
        };

        let binary_operation_kind = match operator_token.kind {
            TokenKind::Asterisk => BinaryOperationKind::Multiply,
            TokenKind::ForwardSlash => BinaryOperationKind::Divide,

            // The next token is not compatible with a binary operation.
            _ => return Ok(left),
        };

        // We can consume the operation token now.
        self.stream.consume_or_err()?;

        let right = self.parse_expression()?;
        let span = SourceSpan::between(&left.span, &right.span);

        Ok(ExpressionNode::from(
            BinaryOperation::new(binary_operation_kind, left, right),
            span,
        ))
    }

    /// Attempts to parse an equals binary operation at the [ASTParser]'s current position.
    fn parse_equals_binary_operation(&mut self) -> Result<ExpressionNode> {
        let left = self.parse_value()?;

        let operator_token = match self.stream.peek() {
            Some(token) => *token,
            None => return Ok(left),
        };

        let binary_operation_kind = match operator_token.kind {
            TokenKind::Equals => {
                self.stream.consume_or_err()?;

                // The next token must also be an equals.
                self.stream.expect(TokenKind::Equals)?;

                BinaryOperationKind::Equals
            }

            TokenKind::ExclamationMark => {
                self.stream.consume_or_err()?;

                // The next token must also be an equals.
                self.stream.expect(TokenKind::Equals)?;

                BinaryOperationKind::NotEquals
            }

            // The next token is not compatible with a binary operation.
            _ => return Ok(left),
        };

        let right = self.parse_expression()?;
        let span = SourceSpan::between(&left.span, &right.span);

        Ok(ExpressionNode::from(
            BinaryOperation::new(binary_operation_kind, left, right),
            span,
        ))
    }

    /// Attempts to parse a single value from the [ASTParser]'s current position.
    fn parse_value(&mut self) -> Result<ExpressionNode> {
        let token = *self.stream.peek_or_err()?;

        // If the token is a left parenthesis, then we can consume the expression within the parenthesis and expect a closing parenthesis.
        if token.kind == TokenKind::LeftParenthesis {
            self.stream.consume_or_err()?;

            let expression = self.parse_expression()?;

            self.stream.expect(TokenKind::RightParenthesis)?;

            return Ok(expression);
        }

        let (kind, span) = match token.kind {
            TokenKind::Ampersand => {
                self.stream.consume_or_err()?;

                // If an ampersand is reached, then we are taking a reference to another trivial value type.
                let inner_value = self.parse_value()?;

                let span = SourceSpan::between(&token.span, &inner_value.span);
                (Reference::new(inner_value).into(), span)
            }

            TokenKind::IntegerLiteral(value) => (IntegerLiteral::new(value).into(), self.stream.consume_or_err()?.span),

            TokenKind::StringLiteral(value) => (StringLiteral::new(value).into(), self.stream.consume_or_err()?.span),

            TokenKind::Identifier(_) if self.stream.nth_is(1, TokenKind::LeftParenthesis) => {
                let (function_call, span) = self.parse_function_call()?;
                (function_call.into(), span)
            }

            TokenKind::Identifier(identifier) => (
                IdentifierReference::new(identifier).into(),
                self.stream.consume_or_err()?.span,
            ),

            TokenKind::Keyword(Keyword::True) => (BooleanLiteral::new(true).into(), self.stream.consume_or_err()?.span),
            TokenKind::Keyword(Keyword::False) => {
                (BooleanLiteral::new(false).into(), self.stream.consume_or_err()?.span)
            }

            _ => return ASTError::unexpected_token(token).into(),
        };

        Ok(ExpressionNode::new(kind, span))
    }

    /// Attempts to parse a function declaration statement from the [ASTParser]'s current position.
    fn parse_function_declaration(&mut self) -> Result<(FunctionDeclaration, SourceSpan)> {
        let starting_span = self.stream.peek_or_err()?.span;

        // If the current token is not the function keyword, we can attempt to parse a modifier keyword.
        let mut modifiers: Vec<FunctionModifier> = vec![];

        while !self.stream.next_is(TokenKind::Keyword(Keyword::Func)) {
            let token = self.stream.consume_or_err()?;

            let modifier = match token.kind {
                TokenKind::Keyword(Keyword::Extern) => FunctionModifier::External,

                _ => return ASTError::expected_token(TokenKind::Keyword(Keyword::Func), *token).into(),
            };

            modifiers.push(modifier);
        }

        // The next token will be the func keyword.
        self.stream.expect(TokenKind::Keyword(Keyword::Func))?;

        // The next token will be the name of the function.
        let (name_reference, name_span) = self.stream.expect_identifier()?;

        // The next token will be a left parenthesis, and then we must parse the parameters.
        self.stream.expect(TokenKind::LeftParenthesis)?;

        let mut parameters: Vec<FunctionParameter> = vec![];

        while !self.stream.next_is(TokenKind::RightParenthesis) {
            // The first token must be the parameter's name.
            let (parameter_name_reference, parameter_name_span) = self.stream.expect_identifier()?;

            // The next token must be a colon.
            self.stream.expect(TokenKind::Colon)?;

            // The next token must be the type.
            let r#type = self.parse_type_reference()?;

            parameters.push(FunctionParameter::new(
                parameter_name_reference,
                r#type,
                SourceSpan::between(&parameter_name_span, &r#type.span),
            ));

            if self.stream.next_is(TokenKind::RightParenthesis) {
                break;
            }

            self.stream.expect(TokenKind::Comma)?;
        }

        let right_parenthesis = *self.stream.expect(TokenKind::RightParenthesis)?;

        // If there is a hyphen, then there is an explicit return type.
        let return_type = if self.stream.next_is(TokenKind::Hyphen) {
            self.stream.consume_or_err()?;
            self.stream.expect(TokenKind::RightAngleBracket)?;

            self.parse_type_reference()?
        } else {
            let type_id = self.type_pool.allocate(Type::Resolved(ResolvedType::Void));
            TypeReference::new(type_id, right_parenthesis.span)
        };

        // If this is an external function, then the function has no body.
        let mut body: Vec<StatementNode> = vec![];

        if modifiers.contains(&FunctionModifier::External) {
            self.stream.expect(TokenKind::Semicolon)?;
        } else {
            // Otherwise, we can parse the function body.
            self.stream.expect(TokenKind::LeftBrace)?;

            while !self.stream.next_is(TokenKind::RightBrace) {
                body.push(self.parse_statement()?);
            }

            self.stream.expect(TokenKind::RightBrace)?;
        }

        Ok((
            FunctionDeclaration::new(name_reference, modifiers, parameters, return_type, body),
            SourceSpan::between(&starting_span, &name_span),
        ))
    }

    /// Attempts to parse an import statement from the [ASTParser]'s current position.
    fn parse_import(&mut self) -> Result<(Import, SourceSpan)> {
        // The first token must be the import keyword.
        let import_token = *self.stream.expect(TokenKind::Keyword(Keyword::Import))?;

        // The next token must be the module name.
        let (module_name_reference, module_name_span) = self.stream.expect_identifier()?;

        Ok((
            Import::new(module_name_reference),
            SourceSpan::between(&import_token.span, &module_name_span),
        ))
    }

    /// Attempts to parse a return statement from the [ASTParser]'s current position.
    fn parse_return(&mut self) -> Result<(Return, SourceSpan)> {
        let return_token = *self.stream.expect(TokenKind::Keyword(Keyword::Return))?;

        // If the next token is a semicolon, then there is no value being returned.
        if self.stream.next_is(TokenKind::Semicolon) {
            self.stream.consume_or_err()?;

            return Ok((Return::empty(), return_token.span));
        }

        let value = self.parse_expression()?;
        let span = SourceSpan::between(&return_token.span, &value.span);

        return Ok((Return::new(value), span));
    }

    /// Attempts to parse a variable declaration from the [ASTParser]'s current position.
    fn parse_variable_declaration(&mut self) -> Result<(VariableDeclaration, SourceSpan)> {
        // The first token must be the type of the variable.
        let r#type = self.parse_type_reference()?;

        // The next token must be the name of the variable.
        let (name_reference, _) = self.stream.expect_identifier()?;

        // The next token must be an equals.
        self.stream.expect(TokenKind::Equals)?;

        // The last token(s) must make an expression;
        let expression = self.parse_expression()?;
        let span = SourceSpan::between(&expression.span, &r#type.span);

        Ok((VariableDeclaration::new(name_reference, r#type, expression), span))
    }

    /// Attempts to parse a variable assignment from the [ASTParser]'s current position.
    fn parse_variable_assignment(&mut self) -> Result<(VariableAssignment, SourceSpan)> {
        // The first token must be the name of the variable.
        let (name_reference, name_span) = self.stream.expect_identifier()?;

        // The next token must be an equals.
        self.stream.expect(TokenKind::Equals)?;

        // The last token(s) must make an expression;
        let expression = self.parse_expression()?;
        let span = SourceSpan::between(&expression.span, &name_span);

        Ok((VariableAssignment::new(name_reference, expression), span))
    }

    /// Attempts to parse a function acll from the [ASTParser]'s current position.
    fn parse_function_call(&mut self) -> Result<(FunctionCall, SourceSpan)> {
        // The first token must be the name of the function being called.
        let (name_reference, name_span) = self.stream.expect_identifier()?;

        // The second token must be a left parenthesis, and before the right parenthesis there may be any number of
        // expressions seperated by commas.
        self.stream.expect(TokenKind::LeftParenthesis)?;

        let mut arguments: Vec<ExpressionNode> = vec![];

        while !self.stream.next_is(TokenKind::RightParenthesis) {
            arguments.push(self.parse_expression()?);

            if self.stream.next_is(TokenKind::Comma) {
                self.stream.consume_or_err()?;
            }
        }

        let right_parenthesis_token = self.stream.expect(TokenKind::RightParenthesis)?;

        Ok((
            FunctionCall::new(name_reference, arguments),
            SourceSpan::between(&name_span, &right_parenthesis_token.span),
        ))
    }

    /// Attempts to parse an if-statement from the [ASTParser]'s current position.
    fn parse_if(&mut self) -> Result<(If, SourceSpan)> {
        // The first token must be the if keyword.
        let if_keyword = *self.stream.expect(TokenKind::Keyword(Keyword::If))?;

        // The next token(s) must make up an expression for the condition.
        let condition = self.parse_expression()?;

        // The next token must be an opening brace.
        let left_brace_token = *self.stream.expect(TokenKind::LeftBrace)?;

        // Then, some statements must make up the body of the if-statement.
        let mut then_block: Vec<StatementNode> = vec![];

        while !self.stream.next_is(TokenKind::RightBrace) {
            then_block.push(self.parse_statement()?);
        }

        self.stream.expect(TokenKind::RightBrace)?;

        // There may be an else block.
        let mut else_block: Vec<StatementNode> = vec![];

        if self.stream.next_is(TokenKind::Keyword(Keyword::Else)) {
            self.stream.consume_or_err()?;

            self.stream.expect(TokenKind::LeftBrace)?;

            while !self.stream.next_is(TokenKind::RightBrace) {
                else_block.push(self.parse_statement()?);
            }

            self.stream.expect(TokenKind::RightBrace)?;
        }

        Ok((
            If::new(condition, then_block, else_block),
            SourceSpan::between(&if_keyword.span, &left_brace_token.span),
        ))
    }

    /// Attempts to parse a while loop from the [ASTParser]'s current position.
    fn parse_while(&mut self) -> Result<(WhileLoop, SourceSpan)> {
        // The first token must be the while keyword.
        let while_keyword = *self.stream.expect(TokenKind::Keyword(Keyword::While))?;

        // The next token(s) must make up an expression for the condition.
        let condition = self.parse_expression()?;

        // The next token must be an opening brace.
        let left_brace_token = *self.stream.expect(TokenKind::LeftBrace)?;

        // Then, some statements must make up the body of the loop.
        let mut block: Vec<StatementNode> = vec![];

        while !self.stream.next_is(TokenKind::RightBrace) {
            block.push(self.parse_statement()?);
        }

        self.stream.expect(TokenKind::RightBrace)?;

        Ok((
            WhileLoop::new(condition, block),
            SourceSpan::between(&while_keyword.span, &left_brace_token.span),
        ))
    }

    /// Attempts to parse a [TypeReference] from the [ASTParser]'s current position.
    fn parse_type_reference(&mut self) -> Result<TypeReference> {
        // If the token is an ampersand, then we have a reference to another type.
        if self.stream.next_is(TokenKind::Ampersand) {
            let amperand_token = *self.stream.consume_or_err()?;
            let reference = self.parse_type_reference()?;

            let type_id = self
                .type_pool
                .allocate(Type::Resolved(ResolvedType::Reference(reference.id)));

            return Ok(TypeReference::new(
                type_id,
                SourceSpan::between(&amperand_token.span, &reference.span),
            ));
        }

        let (identifier_reference, identifier_span) = self.stream.expect_identifier()?;
        let type_id = self.type_pool.allocate(Type::Unresolved(identifier_reference));

        Ok(TypeReference::new(type_id, identifier_span))
    }
}
