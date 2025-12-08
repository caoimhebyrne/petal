use petal_core::{string_intern::StringReference, r#type::TypeReference};

use crate::{expression::ExpressionNode, statement::StatementNodeKind};

/// A variable declaration
#[derive(Debug, PartialEq, Clone)]
pub struct VariableDeclaration {
    /// The name of the variable being declared.
    pub name: StringReference,

    /// The type of the variable.
    pub r#type: TypeReference,

    /// The value being assigned to the variable upon initialization.
    pub value: ExpressionNode,
}

impl VariableDeclaration {
    /// Instantiates a new [VariableDeclaration].
    pub fn new(name: StringReference, r#type: TypeReference, value: ExpressionNode) -> Self {
        VariableDeclaration { name, r#type, value }
    }
}

impl From<VariableDeclaration> for StatementNodeKind {
    fn from(val: VariableDeclaration) -> Self {
        StatementNodeKind::VariableDeclaration(val)
    }
}
