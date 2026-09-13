use petal_span::Span;

/// A statement.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Statement {}

/// A block of [`Statement`]s.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Block {
    /// The [`Statement`]s in this block.
    pub statements: Vec<Statement>,

    /// The [`Span`] that the block covers.
    pub span: Span,
}
