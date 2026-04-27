use crate::ast::{
    expression::Expression,
    statement::StatementKind,
};

/// A variable assignment statement.
#[derive(Debug, Clone, PartialEq)]
pub struct VariableAssignment {
    /// The name of the variable that the value is being assigned to.
    pub name: String,

    /// The value being assigned to the variable.
    pub value: Box<Expression>,
}

impl VariableAssignment {
    /// Creates a new [`VariableAssignment`].
    pub fn new(name: impl Into<String>, value: Expression) -> Self {
        Self { name: name.into(), value: value.into() }
    }
}

impl From<VariableAssignment> for StatementKind {
    fn from(value: VariableAssignment) -> Self {
        Self::VariableAssignment(value)
    }
}
