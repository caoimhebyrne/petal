use crate::value::Value;

/// A binary operation between two values.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BinaryOperation {
    /// The value on the left-hand side.
    pub lhs: Box<Value>,

    /// The value on the right-hand side.
    pub rhs: Box<Value>,

    /// The operand between the two values.
    pub operand: Operand,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Operand {
    Add,
    Subtract,
    Multiply,
    Divide,
}
