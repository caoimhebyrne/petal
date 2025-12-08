use crate::{expression::ExpressionNode, statement::StatementNodeKind};

/// A return statement.
#[derive(Debug, PartialEq, Clone)]
pub struct Return {
    /// The value being returned.
    pub value: Option<ExpressionNode>,
}

impl Return {
    /// Instantiates an empty [ReturnStatement].
    pub fn empty() -> Self {
        Return { value: None }
    }

    /// Instantiates a new [ReturnStatement] with a certain value.
    pub fn new(value: ExpressionNode) -> Self {
        Return { value: Some(value) }
    }
}

impl From<Return> for StatementNodeKind {
    fn from(val: Return) -> Self {
        StatementNodeKind::Return(val)
    }
}
