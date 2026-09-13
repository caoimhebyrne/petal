use petal_span::Span;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Expression {
    IntegerLiteral { value: i64, span: Span },
}

impl Expression {
    /// Return the [`Span`] that corresponds to the expresion.
    pub fn span(&self) -> Span {
        match self {
            Self::IntegerLiteral { span, .. } => *span,
        }
    }
}
