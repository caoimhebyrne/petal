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

    /// The operator to use on the two expressions.
    pub operator: BinaryOperator,
}

impl BinaryOperation {
    /// Creates a new [`BinaryOperation`].
    pub fn new(left: Expression, right: Expression, operator: BinaryOperator) -> Self {
        Self { left: left.into(), right: right.into(), operator }
    }
}

impl From<BinaryOperation> for ExpressionKind {
    fn from(value: BinaryOperation) -> Self {
        Self::BinaryOperation(value)
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum BinaryOperator {
    Add,
    Subtract,
    Divide,
    Multiply,
    Equals,
    NotEquals,
}

impl BinaryOperator {
    pub fn precedence(&self) -> u8 {
        match self {
            Self::Add | Self::Subtract => 1,
            Self::Multiply | Self::Divide => 2,
            _ => panic!(),
        }
    }
}
