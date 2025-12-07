use crate::{
    expression::ExpressionNode,
    statement::{StatementNode, StatementNodeKind},
};

/// An if statement.
#[derive(Debug, PartialEq, Clone)]
pub struct If {
    /// The condition guarding the if statement.
    pub condition: ExpressionNode,

    /// The block of code to execute if the [condition] is true.
    pub then_block: Vec<StatementNode>,

    /// The block of code to execute if the [condition] is false.
    pub else_block: Vec<StatementNode>,
}

impl If {
    pub fn new(condition: ExpressionNode, then_block: Vec<StatementNode>, else_block: Vec<StatementNode>) -> Self {
        If {
            condition,
            then_block,
            else_block,
        }
    }
}

impl Into<StatementNodeKind> for If {
    fn into(self) -> StatementNodeKind {
        StatementNodeKind::If(self)
    }
}
