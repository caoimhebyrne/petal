use crate::{
    expression::ExpressionNode,
    statement::{StatementNode, StatementNodeKind},
};

#[derive(Debug, Clone, PartialEq)]
pub struct WhileLoop {
    /// The condition to evaluate on each iteration of the loop.
    pub condition: ExpressionNode,

    /// The block to execute while the provided condition is true.
    pub block: Vec<StatementNode>,
}

impl WhileLoop {
    pub fn new(condition: ExpressionNode, block: Vec<StatementNode>) -> Self {
        WhileLoop { condition, block }
    }
}

impl Into<StatementNodeKind> for WhileLoop {
    fn into(self) -> StatementNodeKind {
        StatementNodeKind::WhileLoop(self)
    }
}
