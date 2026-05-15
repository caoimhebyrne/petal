use crate::ast::{
    expression::Expression,
    statement::StatementKind,
    type_expr::TypeExpr,
};

/// A variable declaration statement.
#[derive(Debug, Clone, PartialEq)]
pub struct VariableDeclaration {
    /// The name of the variable being declared.
    pub name: String,

    /// The type of the variable being declared.
    pub type_expr: TypeExpr,

    /// The initial value being assigned to the variable.
    pub value: Expression,
}

impl VariableDeclaration {
    /// Creates a new [`VariableDeclaration`].
    pub fn new(name: impl Into<String>, type_expr: TypeExpr, value: Expression) -> Self {
        Self { name: name.into(), type_expr, value }
    }
}

impl From<VariableDeclaration> for StatementKind {
    fn from(value: VariableDeclaration) -> Self {
        Self::VariableDeclaration(value)
    }
}
