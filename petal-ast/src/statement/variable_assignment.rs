use petal_core::string_intern::StringReference;

use crate::{expression::ExpressionNode, statement::StatementNodeKind};

/// A variable declaration
#[derive(Debug, PartialEq, Clone)]
pub struct VariableAssignment {
    /// The name of the variable to assign a new value to.
    pub name: StringReference,

    /// The value being assigned to the variable.
    pub value: ExpressionNode,
}

impl VariableAssignment {
    /// Instantiates a new [VariableAssignment].
    pub fn new(name: StringReference, value: ExpressionNode) -> Self {
        VariableAssignment { name, value }
    }
}

impl From<VariableAssignment> for StatementNodeKind {
    fn from(val: VariableAssignment) -> Self {
        StatementNodeKind::VariableAssignment(val)
    }
}
