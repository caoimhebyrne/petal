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
    pub block: Vec<Statement>,
}

impl If {
    /// Creates a new [`If`].
    pub fn new(condition: Expression, block: Vec<Statement>) -> Self {
        Self { condition: condition.into(), block }
    }
}

impl From<If> for StatementKind {
    fn from(value: If) -> Self {
        Self::If(value)
    }
}
