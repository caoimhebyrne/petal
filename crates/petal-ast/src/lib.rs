mod definition;
mod expression;
mod statement;

pub use definition::*;
pub use expression::*;
use petal_diagnostic::{Diagnostic, DiagnosticSeverity};
use petal_lexer::{Cursor, Token, TokenCursor, TokenKind};
pub use statement::*;

struct Parser<'a, 'd> {
    source: &'a str,

    /// The iterator to consume tokens from.
    cursor: TokenCursor<'a>,

    /// The [`Vec`] of [`Diagnostic`]s to push to.
    diagnostics: &'d mut Vec<Diagnostic>,
}

impl<'a, 'd> Parser<'a, 'd> {
    /// Create a new [`Parser`] consuming the provided [`Iterator`] of [`Token`]s.
    pub fn new(
        source: &'a str,
        tokens: impl Iterator<Item = Token> + 'a,
        diagnostics: &'d mut Vec<Diagnostic>,
    ) -> Self {
        Self {
            source,
            cursor: TokenCursor::new(tokens),
            diagnostics,
        }
    }
}

impl Parser<'_, '_> {
    /// Return the definition at the parser's current position, advancing its cursor.
    pub fn next_definition(&mut self) -> Option<Definition> {
        self.next_function_definition().map(Definition::Function)
    }

    /// Parse a function definition at the parser's current position.
    fn next_function_definition(&mut self) -> Option<FunctionDefinition> {
        // The first token must be an identifier of `func`.
        let identifier_token = self.cursor.consume_if(|it| it.kind == TokenKind::Identifier)?;
        if identifier_token.span.slice(self.source) != "func" {
            return None;
        }

        // Then, there must be the name of the function.
        let name_token = self.cursor.consume_if(|it| it.kind == TokenKind::Identifier)?;

        // Then, there must be an opening parenthesis, followed by a closing parenthesis
        self.cursor.consume_if(|it| it.kind == TokenKind::OpenParen)?;
        self.cursor.consume_if(|it| it.kind == TokenKind::CloseParen)?;

        // Then, there must be a block.
        let body = self.parse_block()?;
        let span = identifier_token.span.until(body.span);

        Some(FunctionDefinition {
            name: name_token.span.slice(self.source).to_string(),
            return_type_name: None,
            body,
            span,
        })
    }

    /// Parse a block at the parser's current position.
    fn parse_block(&mut self) -> Option<Block> {
        // The block must start with an opening brace.
        let open_brace = self.cursor.consume_if(|it| it.kind == TokenKind::OpenBrace)?;

        let statements: Vec<Statement> = Vec::new();

        // And end with a closing brace.
        let close_brace = self.cursor.consume_if(|it| it.kind == TokenKind::CloseBrace)?;

        Some(Block {
            statements,
            span: open_brace.span.until(close_brace.span),
        })
    }
}

/// The result of calling [`parse`].
#[derive(Default)]
pub struct ParseResult {
    /// The definitions parsed from the source.
    pub definitions: Vec<Definition>,

    /// The diagnostics produced while parsing the source.
    pub diagnostics: Vec<Diagnostic>,
}

impl ParseResult {
    /// Return whether the parsing can continue (i.e. whether there are any error diagnostics in the result).
    pub fn can_continue(&self) -> bool {
        self.diagnostics
            .iter()
            .all(|it| it.severity != DiagnosticSeverity::Error)
    }
}

/// Parse definitions from the provided [`Iterator`] of [`Token`]s.
pub fn parse<'a>(source: &'a str, tokens: impl Iterator<Item = Token> + 'a) -> ParseResult {
    let mut result = ParseResult::default();

    let mut parser = Parser::new(source, tokens, &mut result.diagnostics);
    while let Some(definition) = parser.next_definition() {
        result.definitions.push(definition);
    }

    result
}
