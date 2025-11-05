use petal_core::{error::Result, source_span::SourceSpan, string_intern::StringReference};
use petal_lexer::{
    stream::TokenStream,
    token::{Keyword, Token, TokenKind},
};

use crate::{
    error::ASTErrorKind,
    expression::{BinaryOperation, Expression, ExpressionKind, Operation},
    statement::{
        Statement,
        function_call::FunctionCall,
        function_declaration::{FunctionDeclaration, FunctionParameter},
        r#return::ReturnStatement,
        variable_declaration::VariableDeclaration,
    },
    stream::StatementStream,
    token_stream_ext::TokenStreamExt,
    r#type::Type,
};

pub mod error;
pub mod expression;
pub mod statement;
pub mod stream;
pub mod token_stream_ext;
pub mod r#type;
pub mod visitor;

/// Converts tokens from a [Lexer] into an Abstract Syntax Tree.
pub struct ASTParser {
    /// The token stream to read tokens from.
    token_stream: TokenStream,
}

impl ASTParser {
    /// Creates a new [ASTParser] which reads from the provided [Lexer].
    pub fn new(token_stream: TokenStream) -> Self {
        return ASTParser { token_stream };
    }

    /// Parses the token stream that this parser was created with into a [StatementStream].
    pub fn parse(&mut self) -> Result<StatementStream> {
        let mut statements = Vec::new();

        // While there are still characters left in the token stream, we should try to parse a statement.
        while self.token_stream.has_remaining() {
            statements.push(self.parse_statement()?);
        }

        Ok(StatementStream::new(statements))
    }

    /// Parses a [Statement] at the current position of the [TokenStream].
    fn parse_statement(&mut self) -> Result<Statement> {
        let token = self.token_stream.peek_non_whitespace_or_err()?;

        let (statement_result, expect_semicolon) = match token.kind {
            TokenKind::Keyword(Keyword::Func) => (self.parse_function_declaration_node(), false),
            TokenKind::Keyword(Keyword::Return) => (self.parse_return_statement_node(), true),

            // <name>(
            TokenKind::Identifier(_) if self.token_stream.after_next_is(TokenKind::LeftParenthesis) => {
                (self.parse_function_call_statement(), true)
            }

            // <type> <identifier> =
            //   0         1       2
            TokenKind::Identifier(_) if self.token_stream.nth_is(2, TokenKind::Equals) => {
                (self.parse_variable_declaration_node(), true)
            }

            _ => return ASTErrorKind::expected_statement(token).into(),
        };

        // If the parsed statement must end in a semicolon, we can expect one to be present.
        if expect_semicolon {
            self.expect_token(TokenKind::Semicolon)?;
        }

        statement_result
    }

    /// Parses an [Expression] at the current position of the [TokenStream].
    /// This always delegates to [ASTParser::parse_plus_minus_binop].
    fn parse_expression(&mut self) -> Result<Expression> {
        self.parse_plus_minus_binop()
    }

    /// Attempts to parse an addition or subtraction binary operation [Expression] at teh current position of the
    /// [TokenStream].
    ///
    /// If the expression is not a binary operation, the standard [Expression] will be returned.
    fn parse_plus_minus_binop(&mut self) -> Result<Expression> {
        let left_value = self.parse_multiplication_division_binop()?;

        let operation = match self.token_stream.peek_non_whitespace().map(|it| it.kind) {
            Some(TokenKind::Plus) => Operation::Add,
            Some(TokenKind::Hyphen) => Operation::Subtract,

            // There was no binary operator, we can just return the value.
            _ => return Ok(left_value),
        };

        // We can consume the token now that we know it is operator token.
        self.token_stream.consume();

        // We can then get the value on the right-hand side of the expression.
        let right_value = self.parse_expression()?;

        let source_span = SourceSpan::between(&left_value.span, &right_value.span);

        return Ok(Expression::new(
            ExpressionKind::BinaryOperation(BinaryOperation::new(left_value, right_value, operation)),
            source_span,
        ));
    }

    /// Attempts to parse a multiplication or division binary operation [Expression] at the current position of the
    /// [TokenStream].
    ///
    /// If the expression is not a binary operation, the standard [Expression] will be returned.
    fn parse_multiplication_division_binop(&mut self) -> Result<Expression> {
        let left_value = self.parse_value()?;

        let operation = match self.token_stream.peek_non_whitespace().map(|it| it.kind) {
            Some(TokenKind::Asterisk) => Operation::Multiply,
            Some(TokenKind::ForwardSlash) => Operation::Divide,

            // There was no binary operator, we can just return the value.
            _ => return Ok(left_value),
        };

        // We can consume the token now that we are interested in it.
        self.token_stream.consume();

        // We can then get the value on the right-hand side of the expression.
        let right_value = self.parse_expression()?;

        let source_span = SourceSpan::between(&left_value.span, &right_value.span);

        return Ok(Expression::new(
            ExpressionKind::BinaryOperation(BinaryOperation::new(left_value, right_value, operation)),
            source_span,
        ));
    }

    /// Expects a "raw value" to be at the current position of the [TokenStream].
    ///
    /// See also: [ASTParser::parse_expression].
    fn parse_value(&mut self) -> Result<Expression> {
        // If the next token is a left parenthesis, we can attempt to parse the expression within the parenthesis.
        if self.token_stream.next_is(TokenKind::LeftParenthesis) {
            self.expect_token(TokenKind::LeftParenthesis)?;

            let expression = self.parse_expression()?;

            self.expect_token(TokenKind::RightParenthesis)?;

            return Ok(expression);
        }

        let token = *self.token_stream.peek_non_whitespace_or_err()?;

        let expression_kind = match token.kind {
            TokenKind::Identifier(reference) if self.token_stream.after_next_is(TokenKind::LeftParenthesis) => {
                return self.parse_function_call_expression();
            }

            TokenKind::IntegerLiteral(literal) => ExpressionKind::IntegerLiteral(literal),
            TokenKind::Identifier(reference) => ExpressionKind::IdentifierReference(reference),

            _ => return ASTErrorKind::expected_expression(&token).into(),
        };

        // If we got this far, that means that we need to consume the token. Any "advanced" expressions will have
        // returned by now.
        self.token_stream.consume_non_whitespace_or_err()?;

        Ok(Expression::new(expression_kind, token.span))
    }

    /// Attempts to parse a variable declaration node at the current position.
    fn parse_variable_declaration_node(&mut self) -> Result<Statement> {
        // The start of a variable declaration must always start with the type identifier.
        let (type_reference, type_token) = self.expect_identifier()?;

        // The next token must be an identifier.
        let (identifier_reference, _) = self.expect_identifier()?;

        // The next token must be an equals.
        self.expect_token(TokenKind::Equals)?;

        // And finally, an expression must be provided for the initial value.
        let value = self.parse_expression()?;

        let span = SourceSpan::between(&type_token.span, &value.span);

        Ok(Statement::new(
            VariableDeclaration::new(
                identifier_reference,
                Type::unresolved(type_reference, type_token.span),
                value,
            ),
            span,
        ))
    }

    /// Attempts to parse a function declaration node at the current position.
    fn parse_function_declaration_node(&mut self) -> Result<Statement> {
        // The start of a function declaration must always start with the `func` keyword.
        let func_token = self.expect_token(TokenKind::Keyword(Keyword::Func))?;

        // The next token must be an identifier.
        let (identifier_reference, _) = self.expect_identifier()?;

        // The next token must be an opening parenthesis.
        self.expect_token(TokenKind::LeftParenthesis)?;

        let mut parameters = Vec::new();

        // If the next token is not a right parenthesis, we can assume that the token is the start of the first
        // parameter.
        if !self.token_stream.next_is(TokenKind::RightParenthesis) {
            loop {
                // The first token in a parameter must be an identifier which holds its name.
                let (identifier_reference, identifier_token) = self.expect_identifier()?;

                // The next token must be a colon.
                self.expect_token(TokenKind::Colon)?;

                // The next token must be the type of the parameter.
                let (type_identifier, type_token) = self.expect_identifier()?;
                let value_type = Type::unresolved(type_identifier, type_token.span);

                parameters.push(FunctionParameter::new(
                    identifier_reference,
                    value_type,
                    SourceSpan::between(&identifier_token.span, &type_token.span),
                ));

                // If the next token is a comma, we can consume it and continue the loop.
                if self.token_stream.next_is(TokenKind::Comma) {
                    self.expect_token(TokenKind::Comma)?;
                    continue;
                }

                // Otherwise, we have reached the end of the parameter list.
                break;
            }
        }

        // After the parameters, there must be a closing parenthesis.
        let right_parenthesis_token = self.expect_token(TokenKind::RightParenthesis)?;

        // There might be a hyphen, and if there is, we can attempt to parse the return type.
        let mut return_type = Type::void(right_parenthesis_token.span);

        if self.token_stream.next_is(TokenKind::Hyphen) {
            // We can consume the hyphen token.
            self.expect_token(TokenKind::Hyphen)?;

            // And then, there must be a right angle bracket.
            self.expect_token(TokenKind::RightAngleBracket)?;

            // And finally, there must be an identifier for the function's return type.
            let (return_type_identifier, return_type_token) = self.expect_identifier()?;
            return_type = Type::unresolved(return_type_identifier, return_type_token.span);
        }

        // We can then consume statements until we find a closing brace.
        let left_brace_token = self.expect_token(TokenKind::LeftBrace)?;

        let mut body: Vec<Statement> = Vec::new();

        while !self.token_stream.next_is(TokenKind::RightBrace) {
            body.push(self.parse_statement()?);
        }

        self.expect_token(TokenKind::RightBrace)?;

        Ok(Statement::new(
            FunctionDeclaration::new(identifier_reference, parameters, return_type, body),
            SourceSpan::between(&func_token.span, &left_brace_token.span),
        ))
    }

    /// Attempts to parse a return statement node at the current position.
    fn parse_return_statement_node(&mut self) -> Result<Statement> {
        // The first token must be the return keyword.
        let return_token = self.expect_token(TokenKind::Keyword(Keyword::Return))?;

        // If the next token is not a semicolon, then there should be a return value present.
        let mut value: Option<Expression> = None;
        if !self.token_stream.next_is(TokenKind::Semicolon) {
            value = Some(self.parse_expression()?);
        }

        // If a value was present, we can use its span as the end span, otherwise, we can just use the return token.
        let end_span = value.as_ref().map(|it| it.span).unwrap_or(return_token.span);

        Ok(Statement::new(
            ReturnStatement::new(value),
            SourceSpan::between(&return_token.span, &end_span),
        ))
    }

    /// Attempts to parse a function call at the current position.
    fn parse_function_call(&mut self) -> Result<(FunctionCall, SourceSpan)> {
        // The first token must be an identifier.
        let (identifier_reference, identifier_token) = self.expect_identifier()?;

        // The next token must be an opening parenthesis.
        self.expect_token(TokenKind::LeftParenthesis)?;

        // Then, we can attempt to parse the arguments.
        let mut arguments = vec![];

        while !self.token_stream.next_is(TokenKind::RightParenthesis) {
            arguments.push(self.parse_expression()?);

            // If the next token is a comma, we can consume it and continue the loop.
            if self.token_stream.next_is(TokenKind::Comma) {
                self.expect_token(TokenKind::Comma)?;
                continue;
            }
        }

        // The next token must be a closing parenthesis.
        let closing_parenthesis = self.expect_token(TokenKind::RightParenthesis)?;

        Ok((
            FunctionCall::new(identifier_reference, arguments),
            SourceSpan::between(&identifier_token.span, &closing_parenthesis.span),
        ))
    }

    /// Attempts to parse a function call statement at the current position.
    fn parse_function_call_statement(&mut self) -> Result<Statement> {
        let (function_call, source_span) = self.parse_function_call()?;

        Ok(Statement::new(function_call, source_span))
    }

    /// Attempts to parse a function call expression at the current position.
    fn parse_function_call_expression(&mut self) -> Result<Expression> {
        let (function_call, source_span) = self.parse_function_call()?;

        Ok(Expression::new(
            ExpressionKind::FunctionCall(function_call),
            source_span,
        ))
    }

    /// Expects a certain [TokenKind] to be produced by the lexer, returning an [Err] if a different token was returned.
    fn expect_token(&mut self, kind: TokenKind) -> Result<Token> {
        // If the token's kind does not match, we can return an error.
        let token = self.token_stream.consume_non_whitespace_or_err()?;
        if token.kind != kind {
            return ASTErrorKind::expected_token(kind, token).into();
        }

        Ok(*token)
    }

    /// Expects an identifier token to be produced by the lexer, returning an [Err] if a different token was returned.
    fn expect_identifier(&mut self) -> Result<(StringReference, Token)> {
        let token = self.token_stream.consume_non_whitespace_or_err()?;
        match token.kind {
            TokenKind::Identifier(reference) => Ok((reference, *token)),

            _ => ASTErrorKind::expected_identifier(token).into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use petal_core::string_intern::{StringInternPool, StringInternPoolImpl};
    use petal_lexer::Lexer;

    // Note this useful idiom: importing names from outer (for mod tests) scope.\
    use super::*;
    use crate::r#type::Type;

    #[test]
    fn test_variable_declaration() {
        let mut string_intern_pool = StringInternPoolImpl::new();
        let type_reference = StringReference(0);
        let identifier_reference = StringReference(1);

        let mut lexer = Lexer::new(&mut string_intern_pool, "i32 identifier = 123456;");
        let token_stream = lexer.get_stream().expect("get_stream should not fail");

        let mut ast_parser = ASTParser::new(token_stream);

        assert_eq!(
            ast_parser.parse_statement().expect("next_statement should not fail!"),
            Statement {
                kind: VariableDeclaration::new(
                    identifier_reference,
                    Type::unresolved(type_reference, SourceSpan { start: 0, end: 3 }),
                    Expression {
                        kind: ExpressionKind::IntegerLiteral(123456),
                        r#type: None,
                        span: SourceSpan { start: 17, end: 23 }
                    }
                )
                .into(),
                span: SourceSpan { start: 0, end: 23 }
            }
        );

        assert_eq!(string_intern_pool.resolve_reference(&type_reference), Some("i32"));

        assert_eq!(
            string_intern_pool.resolve_reference(&identifier_reference),
            Some("identifier")
        );
    }

    #[test]
    fn test_empty_function_declaration() {
        let mut string_intern_pool = StringInternPoolImpl::new();
        let function_name_reference = StringReference(0);

        let mut lexer = Lexer::new(&mut string_intern_pool, "func test() {}");
        let token_stream = lexer.get_stream().expect("get_stream should not fail");

        let mut ast_parser = ASTParser::new(token_stream);

        assert_eq!(
            ast_parser.parse_statement().expect("next_statement should not fail!"),
            Statement {
                kind: FunctionDeclaration::new(
                    function_name_reference,
                    vec![],
                    Type::void(SourceSpan { start: 10, end: 11 }),
                    vec![]
                )
                .into(),
                span: SourceSpan { start: 0, end: 13 }
            }
        );

        assert_eq!(
            string_intern_pool.resolve_reference(&function_name_reference),
            Some("test")
        );
    }

    #[test]
    fn test_function_declaration() {
        let mut string_intern_pool = StringInternPoolImpl::new();
        let function_name_reference = StringReference(0);
        let type_reference = StringReference(1);
        let identifier_reference = StringReference(2);

        let mut lexer = Lexer::new(&mut string_intern_pool, "func test() { i32 i = 4; }");
        let token_stream = lexer.get_stream().expect("get_stream should not fail");

        let mut ast_parser = ASTParser::new(token_stream);

        assert_eq!(
            ast_parser.parse_statement().expect("next_statement should not fail!"),
            Statement {
                kind: FunctionDeclaration::new(
                    function_name_reference,
                    vec![],
                    Type::void(SourceSpan { start: 10, end: 11 }),
                    vec![Statement {
                        kind: VariableDeclaration::new(
                            identifier_reference,
                            Type::unresolved(type_reference, SourceSpan { start: 14, end: 17 }),
                            Expression {
                                kind: ExpressionKind::IntegerLiteral(4).into(),
                                r#type: None,
                                span: SourceSpan { start: 22, end: 23 }
                            }
                        )
                        .into(),
                        span: SourceSpan { start: 14, end: 23 }
                    }]
                )
                .into(),
                span: SourceSpan { start: 0, end: 13 }
            }
        );

        assert_eq!(
            string_intern_pool.resolve_reference(&function_name_reference),
            Some("test")
        );

        assert_eq!(string_intern_pool.resolve_reference(&type_reference), Some("i32"));
    }

    #[test]
    fn test_empty_function_declaration_with_parameter() {
        let mut string_intern_pool = StringInternPoolImpl::new();
        let function_name_reference = StringReference(0);
        let p0_name_reference = StringReference(1);
        let p0_type_reference = StringReference(2);

        let mut lexer = Lexer::new(&mut string_intern_pool, "func test(name: i32) {}");
        let token_stream = lexer.get_stream().expect("get_stream should not fail");

        let mut ast_parser = ASTParser::new(token_stream);

        assert_eq!(
            ast_parser.parse_statement().expect("next_statement should not fail!"),
            Statement {
                kind: FunctionDeclaration::new(
                    function_name_reference,
                    vec![FunctionParameter::new(
                        p0_name_reference,
                        Type {
                            kind: r#type::TypeKind::Unresolved(p0_type_reference),
                            span: SourceSpan { start: 16, end: 19 }
                        },
                        SourceSpan { start: 10, end: 19 }
                    )],
                    Type::void(SourceSpan { start: 19, end: 20 }),
                    vec![]
                )
                .into(),
                span: SourceSpan { start: 0, end: 22 }
            }
        );

        assert_eq!(
            string_intern_pool.resolve_reference(&function_name_reference),
            Some("test")
        );

        assert_eq!(string_intern_pool.resolve_reference(&p0_name_reference), Some("name"));

        assert_eq!(string_intern_pool.resolve_reference(&p0_type_reference), Some("i32"));
    }

    #[test]
    fn test_empty_function_declaration_with_parameters() {
        let mut string_intern_pool = StringInternPoolImpl::new();
        let function_name_reference = StringReference(0);

        let p0_name_reference = StringReference(1);
        let p0_type_reference = StringReference(2);

        let p1_name_reference = StringReference(3);
        let p1_type_reference = StringReference(2);

        let mut lexer = Lexer::new(&mut string_intern_pool, "func test(name: i32, other: i32) {}");
        let token_stream = lexer.get_stream().expect("get_stream should not fail");

        let mut ast_parser = ASTParser::new(token_stream);

        assert_eq!(
            ast_parser.parse_statement().expect("next_statement should not fail!"),
            Statement {
                kind: FunctionDeclaration::new(
                    function_name_reference,
                    vec![
                        FunctionParameter::new(
                            p0_name_reference,
                            Type {
                                kind: r#type::TypeKind::Unresolved(p0_type_reference),
                                span: SourceSpan { start: 16, end: 19 }
                            },
                            SourceSpan { start: 10, end: 19 }
                        ),
                        FunctionParameter::new(
                            p1_name_reference,
                            Type {
                                kind: r#type::TypeKind::Unresolved(p1_type_reference),
                                span: SourceSpan { start: 28, end: 31 }
                            },
                            SourceSpan { start: 21, end: 31 }
                        )
                    ],
                    Type::void(SourceSpan { start: 31, end: 32 }),
                    vec![]
                )
                .into(),
                span: SourceSpan { start: 0, end: 34 }
            }
        );

        assert_eq!(
            string_intern_pool.resolve_reference(&function_name_reference),
            Some("test")
        );

        assert_eq!(string_intern_pool.resolve_reference(&p0_name_reference), Some("name"));
        assert_eq!(string_intern_pool.resolve_reference(&p0_type_reference), Some("i32"));

        assert_eq!(string_intern_pool.resolve_reference(&p1_name_reference), Some("other"));
        assert_eq!(string_intern_pool.resolve_reference(&p1_type_reference), Some("i32"));
    }

    #[test]
    fn test_function_declaration_with_return_type() {
        let mut string_intern_pool = StringInternPoolImpl::new();
        let function_name_reference = StringReference(0);
        let function_return_type_reference = StringReference(1);

        let mut lexer = Lexer::new(&mut string_intern_pool, "func test() -> i32 {}");
        let token_stream = lexer.get_stream().expect("get_stream should not fail");

        let mut ast_parser = ASTParser::new(token_stream);

        assert_eq!(
            ast_parser.parse_statement().expect("next_statement should not fail!"),
            Statement {
                kind: FunctionDeclaration::new(
                    function_name_reference,
                    vec![],
                    Type::unresolved(function_return_type_reference, SourceSpan { start: 15, end: 18 }),
                    vec![]
                )
                .into(),
                span: SourceSpan { start: 0, end: 20 }
            }
        );

        assert_eq!(
            string_intern_pool.resolve_reference(&function_name_reference),
            Some("test")
        );

        assert_eq!(
            string_intern_pool.resolve_reference(&function_return_type_reference),
            Some("i32")
        );
    }

    #[test]
    fn test_return_void() {
        let mut string_intern_pool = StringInternPoolImpl::new();

        let mut lexer = Lexer::new(&mut string_intern_pool, "return;");
        let token_stream = lexer.get_stream().expect("get_stream should not fail");

        let mut ast_parser = ASTParser::new(token_stream);

        assert_eq!(
            ast_parser.parse_statement().expect("next_statement should not fail!"),
            Statement {
                kind: ReturnStatement::new(None).into(),
                span: SourceSpan { start: 0, end: 6 }
            }
        );
    }

    #[test]
    fn test_return_with_value() {
        let mut string_intern_pool = StringInternPoolImpl::new();

        let mut lexer = Lexer::new(&mut string_intern_pool, "return 123;");
        let token_stream = lexer.get_stream().expect("get_stream should not fail");

        let mut ast_parser = ASTParser::new(token_stream);

        assert_eq!(
            ast_parser.parse_statement().expect("next_statement should not fail!"),
            Statement {
                kind: ReturnStatement::new(Some(Expression {
                    kind: ExpressionKind::IntegerLiteral(123).into(),
                    r#type: None,
                    span: SourceSpan { start: 7, end: 10 }
                }))
                .into(),
                span: SourceSpan { start: 0, end: 10 }
            }
        );
    }
}
