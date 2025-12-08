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

impl From<BooleanLiteral> for ExpressionNodeKind {
    fn from(val: BooleanLiteral) -> Self {
        ExpressionNodeKind::BooleanLiteral(val)
    }
}
