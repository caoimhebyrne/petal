use crate::{
    ast::{
        error::ASTErrorKind,
        expression::{Expression, ExpressionKind},
        statement::{
            Statement, function_declaration::FunctionDeclaration, r#return::ReturnStatement,
            variable_declaration::VariableDeclaration,
        },
        visitor::ASTVisitor,
    },
    core::{
        error::Result,
        source_span::SourceSpan,
        string_intern::{StringInternPool, StringReference},
    },
    lexer::{
        stream::TokenStream,
        token::{Keyword, Token, TokenKind},
    },
};

pub mod error;
pub mod expression;
pub mod statement;
pub mod visitor;

/// Converts tokens from a [Lexer] into an Abstract Syntax Tree.
pub struct ASTParser<'a> {
    /// The string intern pool to read strings from.
    #[allow(dead_code)]
    string_intern_pool: &'a mut dyn StringInternPool,

    /// The token stream to read tokens from.
    token_stream: TokenStream,
}

impl<'a> ASTParser<'a> {
    /// Creates a new [ASTParser] which reads from the provided [Lexer].
    pub fn new(string_intern_pool: &'a mut dyn StringInternPool, token_stream: TokenStream) -> Self {
        return ASTParser {
            string_intern_pool,
            token_stream,
        };
    }

    /// Parses the token stream that this parser was created with into an AST, calling on the provided [ASTVistior]
    /// to consume any nodes.
    pub fn parse(&mut self, visitor: &dyn ASTVisitor) -> Result<()> {
        // While there are still characters left in the token stream, we should try to parse a statement.
        while self.token_stream.has_remaining() {
            let statement = self.next_statement()?;
            visitor.visit(&statement)?;
        }

        Ok(())
    }

    /// Returns the next AST node at the current position in the source code.
    fn next_statement(&mut self) -> Result<Statement> {
        let token = self
            .token_stream
            .peek_non_whitespace()
            .ok_or_else(|| ASTErrorKind::unexpected_end_of_file())?;

        let (statement_result, expect_semicolon) = match token.kind {
            TokenKind::Keyword(Keyword::Let) => (self.parse_variable_declaration_node(), true),
            TokenKind::Keyword(Keyword::Func) => (self.parse_function_declaration_node(), false),
            TokenKind::Keyword(Keyword::Return) => (self.parse_return_statement_node(), true),

            _ => return ASTErrorKind::expected_statement(token).into(),
        };

        // If the parsed statement must end in a semicolon, we can expect one to be present.
        if expect_semicolon {
            self.expect_token(TokenKind::Semicolon)?;
        }

        statement_result
    }

    fn next_expression(&mut self) -> Result<Expression> {
        // The only expression type that is supported is the integer literal.
        let token = self
            .token_stream
            .next_non_whitespace()
            .ok_or_else(|| ASTErrorKind::unexpected_end_of_file())?;

        let integer_literal = match token.kind {
            TokenKind::IntegerLiteral(literal) => literal,
            _ => return ASTErrorKind::unexpected_end_of_file().into(),
        };

        Ok(Expression::new(
            ExpressionKind::IntegerLiteral(integer_literal),
            token.span,
        ))
    }

    /// Attempts to parse a variable declaration node at the current position.
    fn parse_variable_declaration_node(&mut self) -> Result<Statement> {
        // The start of a variable declaration must always start with the `let` keyword.
        let let_token = self.expect_token(TokenKind::Keyword(Keyword::Let))?;

        // The next token must be an identifier.
        let (identifier_reference, _) = self.expect_identifier()?;

        // The next token must be an equals.
        self.expect_token(TokenKind::Equals)?;

        // And finally, an expression must be provided for the initial value.
        let value = self.next_expression()?;

        let span = SourceSpan::between(&let_token.span, &value.span);

        Ok(Statement::new(
            VariableDeclaration::new(identifier_reference, value),
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

        // TODO: Parse parameters.

        // After the parameters, there must be a closing parenthesis.
        self.expect_token(TokenKind::RightParenthesis)?;

        // There might be a hyphen, and if there is, we can attempt to parse the return type.
        if let Some(TokenKind::Hyphen) = self.token_stream.peek_non_whitespace().map(|it| it.kind) {
            // We can consume the hyphen token.
            self.expect_token(TokenKind::Hyphen)?;

            // And then, there must be a right angle bracket.
            self.expect_token(TokenKind::RightAngleBracket)?;

            // And finally, there must be an identifier for the function's return type.
            let (_return_type_identifier, _return_type_token) = self.expect_identifier()?;

            // TODO: Create a `Type` with the reference and span for the type.
        }

        // We can then consume statements until we find a closing brace.
        let left_brace_token = self.expect_token(TokenKind::LeftBrace)?;

        let mut body: Vec<Statement> = Vec::new();

        loop {
            let next_token = self
                .token_stream
                .peek_non_whitespace()
                .ok_or(ASTErrorKind::unexpected_end_of_file())?;

            // If the next token is a closing brace, then we have reached the end of the function body.
            if next_token.kind == TokenKind::RightBrace {
                break;
            }

            // Otherwise, we can attempt to parse a statement and add it to the body.
            body.push(self.next_statement()?);
        }

        self.expect_token(TokenKind::RightBrace)?;

        Ok(Statement::new(
            FunctionDeclaration::new(identifier_reference, body),
            SourceSpan::between(&func_token.span, &left_brace_token.span),
        ))
    }

    /// Attempts to parse a return statement node at the current position.
    fn parse_return_statement_node(&mut self) -> Result<Statement> {
        // The first token must be the return keyword.
        let return_token = self.expect_token(TokenKind::Keyword(Keyword::Return))?;

        // If the next token is not a semicolon, then there should be a value.
        let mut value: Option<Expression> = None;
        if self.token_stream.peek_non_whitespace().map(|it| it.kind) != Some(TokenKind::Semicolon) {
            value = Some(self.next_expression()?);
        }

        // If a value was present, we can use its span as the end span, otherwise, we can just use the return token.
        let end_span = match &value {
            Some(value) => value.span,
            None => return_token.span,
        };

        Ok(Statement::new(
            ReturnStatement::new(value),
            SourceSpan::between(&return_token.span, &end_span),
        ))
    }

    /// Expects a certain [TokenKind] to be produced by the lexer, returning an [Err] if a different token was returned.
    fn expect_token(&mut self, kind: TokenKind) -> Result<Token> {
        let token = self
            .token_stream
            .next_non_whitespace()
            .ok_or_else(|| ASTErrorKind::unexpected_end_of_file())?;

        // If the token's kind does not match, we can return an error.
        if token.kind != kind {
            return ASTErrorKind::expected_token(kind, token).into();
        }

        Ok(*token)
    }

    /// Expects an identifier token to be produced by the lexer, returning an [Err] if a different token was returned.
    fn expect_identifier(&mut self) -> Result<(StringReference, Token)> {
        let token = self
            .token_stream
            .next_non_whitespace()
            .ok_or_else(|| ASTErrorKind::unexpected_end_of_file())?;

        match token.kind {
            TokenKind::Identifier(reference) => Ok((reference, *token)),
            _ => ASTErrorKind::expected_identifier(token).into(),
        }
    }
}

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.\
    use super::*;
    use crate::{
        core::{source_span::SourceSpan, string_intern::StringInternPoolImpl},
        lexer::Lexer,
    };

    #[test]
    fn test_variable_declaration() {
        let mut string_intern_pool = StringInternPoolImpl::new();
        let identifier_reference = StringReference(0);

        let mut lexer = Lexer::new(&mut string_intern_pool, "let identifier = 123456;");
        let token_stream = lexer.get_stream().expect("get_stream should not fail");

        let mut ast_parser = ASTParser::new(&mut string_intern_pool, token_stream);

        assert_eq!(
            ast_parser.next_statement().expect("next_statement should not fail!"),
            Statement {
                kind: VariableDeclaration::new(
                    identifier_reference,
                    Expression {
                        kind: ExpressionKind::IntegerLiteral(123456),
                        span: SourceSpan { start: 17, end: 23 }
                    }
                )
                .into(),
                span: SourceSpan { start: 0, end: 23 }
            }
        );

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

        let mut ast_parser = ASTParser::new(&mut string_intern_pool, token_stream);

        assert_eq!(
            ast_parser.next_statement().expect("next_statement should not fail!"),
            Statement {
                kind: FunctionDeclaration::new(function_name_reference, vec![]).into(),
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
        let identifier_reference = StringReference(1);

        let mut lexer = Lexer::new(&mut string_intern_pool, "func test() { let i = 4; }");
        let token_stream = lexer.get_stream().expect("get_stream should not fail");

        let mut ast_parser = ASTParser::new(&mut string_intern_pool, token_stream);

        assert_eq!(
            ast_parser.next_statement().expect("next_statement should not fail!"),
            Statement {
                kind: FunctionDeclaration::new(
                    function_name_reference,
                    vec![Statement {
                        kind: VariableDeclaration::new(
                            identifier_reference,
                            Expression {
                                kind: ExpressionKind::IntegerLiteral(4).into(),
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
    }

    #[test]
    fn test_return_void() {
        let mut string_intern_pool = StringInternPoolImpl::new();

        let mut lexer = Lexer::new(&mut string_intern_pool, "return;");
        let token_stream = lexer.get_stream().expect("get_stream should not fail");

        let mut ast_parser = ASTParser::new(&mut string_intern_pool, token_stream);

        assert_eq!(
            ast_parser.next_statement().expect("next_statement should not fail!"),
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

        let mut ast_parser = ASTParser::new(&mut string_intern_pool, token_stream);

        assert_eq!(
            ast_parser.next_statement().expect("next_statement should not fail!"),
            Statement {
                kind: ReturnStatement::new(Some(Expression {
                    kind: ExpressionKind::IntegerLiteral(123).into(),
                    span: SourceSpan { start: 7, end: 10 }
                }))
                .into(),
                span: SourceSpan { start: 0, end: 10 }
            }
        );
    }
}
