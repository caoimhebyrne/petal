use crate::ast::expression::{
    Expression,
    ExpressionKind,
};

#[derive(Debug, Clone, PartialEq)]
pub struct BinaryOperation {
    /// The left-hand side of the operation.
    pub left: Box<Expression>,

    /// The right-hand side of the operation.
    pub right: Box<Expression>,

    /// The operand to use on the two expressions.
    pub operand: BinaryOperand,
}

impl BinaryOperation {
    /// Creates a new [`BinaryOperation`].
    pub fn new(left: Expression, right: Expression, operand: BinaryOperand) -> Self {
        Self { left: left.into(), right: right.into(), operand }
    }
}

impl From<BinaryOperation> for ExpressionKind {
    fn from(value: BinaryOperation) -> Self {
        Self::BinaryOperation(value)
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum BinaryOperand {
    Add,
    Subtract,
    Divide,
    Multiply,
}
