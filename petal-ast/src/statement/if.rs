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
    pub block: Vec<StatementNode>,
}

impl If {
    pub fn new(condition: ExpressionNode, block: Vec<StatementNode>) -> Self {
        If { condition, block }
    }
}

impl Into<StatementNodeKind> for If {
    fn into(self) -> StatementNodeKind {
        StatementNodeKind::If(self)
    }
}
