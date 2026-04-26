use crate::ast::{
    expression::Expression,
    statement::StatementKind,
    r#type::Type,
};

/// A variable declaration statement.
#[derive(Debug, Clone, PartialEq)]
pub struct VariableDeclaration {
    /// The name of the variable being declared.
    pub name: String,

    /// The type of the variable being declared.
    pub r#type: Type,

    /// The initial value being assigned to the variable.
    pub value: Expression,
}

impl VariableDeclaration {
    /// Creates a new [`VariableDeclaration`].
    pub fn new(name: impl Into<String>, r#type: Type, value: Expression) -> Self {
        Self { name: name.into(), r#type, value }
    }
}

impl From<VariableDeclaration> for StatementKind {
    fn from(value: VariableDeclaration) -> Self {
        Self::VariableDeclaration(value)
    }
}
