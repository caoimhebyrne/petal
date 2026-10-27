use petal_span::Span;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Expression {
    IntegerLiteral { id: ExpressionId, value: i64, span: Span },
}

impl Expression {
    /// Return the [`Span`] that corresponds to the expresion.
    pub fn span(&self) -> Span {
        match self {
            Self::IntegerLiteral { span, .. } => *span,
        }
    }

    /// Return the [`ExpressionId`] for this expression.
    pub fn id(&self) -> ExpressionId {
        match self {
            Self::IntegerLiteral { id, .. } => *id,
        }
    }
}

/// A unique identifier assigned to an expression.
///
/// IDs are allocated by the parser, starting from zero for each parse, so they can be used to index into a table
/// sized by [`crate::ParseResult::expression_count`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ExpressionId(pub(crate) u32);

impl ExpressionId {
    /// Return the index of this ID, for use in tables indexed by [`ExpressionId`].
    pub const fn index(self) -> usize {
        self.0 as usize
    }
}
