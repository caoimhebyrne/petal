use crate::core::span::Span;

/// An expression node within the abstract syntax tree.
/// This node kind should always emit a value once evaluated.
#[derive(Debug, Clone, PartialEq)]
pub struct Expression {
    /// The kind of expression that this is.
    pub kind: ExpressionKind,

    /// The [Span] that this expression occured at within the source file.
    pub span: Span,
}

impl Expression {
    /// Creates a new [Expression].
    pub fn new(kind: ExpressionKind, span: Span) -> Self {
        Self { kind, span }
    }
}

/// The different kinds of [Expression]s that are available in an abstract syntax tree.
#[derive(Debug, Clone, PartialEq)]
pub enum ExpressionKind {
    /// A number literal.
    NumberLiteral(f64),
}
