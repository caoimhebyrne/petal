use crate::expression::ExpressionNodeKind;

/// A boolean literal.
#[derive(Debug, PartialEq, Clone)]
pub struct BooleanLiteral {
    /// The value of the literal.
    pub value: bool,
}

impl BooleanLiteral {
    /// Instantiates a new [BooleanLiteral].
    pub fn new(value: bool) -> Self {
        BooleanLiteral { value }
    }
}

impl Into<ExpressionNodeKind> for BooleanLiteral {
    fn into(self) -> ExpressionNodeKind {
        ExpressionNodeKind::BooleanLiteral(self)
    }
}
