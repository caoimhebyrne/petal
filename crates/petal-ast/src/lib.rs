mod definition;
mod expression;
mod statement;

pub use definition::*;
pub use expression::*;
use petal_lexer::{Cursor, Token, TokenCursor, TokenKind};
pub use statement::*;

struct Parser<'a> {
    source: &'a str,

    /// The iterator to consume tokens from.
    cursor: TokenCursor<'a>,
}

impl<'a> Parser<'a> {
    /// Create a new [`Parser`] consuming the provided [`Iterator`] of [`Token`]s.
    pub fn new(source: &'a str, tokens: impl Iterator<Item = Token> + 'a) -> Self {
        Self {
            source,
            cursor: TokenCursor::new(tokens),
        }
    }
}

impl Parser<'_> {
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

/// Parse definitions from the provided [`Iterator`] of [`Token`]s.
pub fn parse<'a>(source: &'a str, tokens: impl Iterator<Item = Token> + 'a) -> impl Iterator<Item = Definition> + 'a {
    let mut parser = Parser::new(source, tokens);
    std::iter::from_fn(move || parser.next_definition())
}
