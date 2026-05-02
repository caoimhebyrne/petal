use crate::ast::{
    expression::Expression,
    statement::StatementKind,
};

/// A variable assignment statement.
#[derive(Debug, Clone, PartialEq)]
pub struct VariableAssignment {
    /// The expression containing the variable to assign to.
    pub target: Box<Expression>,

    /// The value being assigned to the variable.
    pub value: Box<Expression>,
}

impl VariableAssignment {
    /// Creates a new [`VariableAssignment`].
    pub fn new(target: Expression, value: Expression) -> Self {
        Self { target: target.into(), value: value.into() }
    }
}

impl From<VariableAssignment> for StatementKind {
    fn from(value: VariableAssignment) -> Self {
        Self::VariableAssignment(value)
    }
}
