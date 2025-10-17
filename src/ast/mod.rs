use crate::{
    ast::{error::ASTErrorKind, node::Node},
    core::{error::Error, source_span::SourceSpan},
    lexer::Lexer,
};

pub mod error;
pub mod node;

/// Converts tokens from a [Lexer] into an Abstract Syntax Tree.
pub struct ASTParser<'a> {
    /// The lexer to read tokens from.
    lexer: &'a mut Lexer<'a>,
}

impl<'a> ASTParser<'a> {
    /// Creates a new [ASTParser] which reads from the provided [Lexer].
    pub fn new(lexer: &'a mut Lexer<'a>) -> ASTParser<'a> {
        return ASTParser { lexer };
    }

    /// Returns the next AST node at the current position in the source code.
    pub fn next_node(&mut self) -> Result<Node, Error> {
        // TODO: Implement.
        Error {
            kind: ASTErrorKind::UnexpectedEndOfFile.into(),
            span: SourceSpan { start: 0, end: 0 },
        }
        .into()
    }
}
