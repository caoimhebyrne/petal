use crate::expression::{ExpressionNode, ExpressionNodeKind};

/// A binary operation between two nodes.
#[derive(Debug, PartialEq, Clone)]
pub struct BinaryOperation {
    /// The kind of [BinaryOperation] that this is.
    pub kind: BinaryOperationKind,

    /// The expression on the left side of the operation.
    pub left: Box<ExpressionNode>,

    /// The expression on the right side of the operation.
    pub right: Box<ExpressionNode>,
}

impl BinaryOperation {
    /// Instantiates a new [BinaryOperation].
    pub fn new(kind: BinaryOperationKind, left: ExpressionNode, right: ExpressionNode) -> Self {
        BinaryOperation {
            kind,
            left: Box::new(left),
            right: Box::new(right),
        }
    }
}

/// The recognized kinds of binary operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOperationKind {
    Add,
    Subtract,
    Multiply,
    Divide,
    Equals,
}

impl Into<ExpressionNodeKind> for BinaryOperation {
    fn into(self) -> ExpressionNodeKind {
        ExpressionNodeKind::BinaryOperation(self)
    }
}
