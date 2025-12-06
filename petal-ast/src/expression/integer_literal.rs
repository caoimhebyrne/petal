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

impl Into<ExpressionNodeKind> for IntegerLiteral {
    fn into(self) -> ExpressionNodeKind {
        ExpressionNodeKind::IntegerLiteral(self)
    }
}
