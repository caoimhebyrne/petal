use crate::expression::ExpressionNodeKind;

/// An integer literal.
#[derive(Debug, PartialEq, Clone)]
pub struct IntegerLiteral {
    /// The value of the integer.
    pub value: u64,
}

impl IntegerLiteral {
    /// Instantiates a new [IntegerLiteral].
    pub fn new(value: u64) -> Self {
        IntegerLiteral { value }
    }
}

impl From<IntegerLiteral> for ExpressionNodeKind {
    fn from(val: IntegerLiteral) -> Self {
        ExpressionNodeKind::IntegerLiteral(val)
    }
}
