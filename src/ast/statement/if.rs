use crate::ast::{
    expression::Expression,
    statement::{
        Statement,
        StatementKind,
    },
};

#[derive(Debug, Clone, PartialEq)]
pub struct If {
    /// The condition to the block.
    pub condition: Box<Expression>,

    /// The block to execute if the [condition] is true.
    pub then_block: Vec<Statement>,

    /// The block to execute if the [condition] is false.
    pub else_block: Vec<Statement>,
}

impl If {
    /// Creates a new [`If`].
    pub fn new(condition: Expression, then_block: Vec<Statement>, else_block: Vec<Statement>) -> Self {
        Self { condition: condition.into(), then_block, else_block }
    }
}

impl From<If> for StatementKind {
    fn from(value: If) -> Self {
        Self::If(value)
    }
}
