use crate::ast::{
    statement::StatementKind,
    type_expr::TypeExpr,
};

#[derive(Debug, Clone, PartialEq)]
pub struct TypeDeclaration {
    /// The name of the type being declared.
    pub name: String,

    /// The type being declared.
    pub type_expr: TypeExpr,
}

impl TypeDeclaration {
    /// Creates a new [`TypeDeclaration`].
    pub fn new(name: String, type_expr: TypeExpr) -> Self {
        Self { name, type_expr }
    }
}

impl From<TypeDeclaration> for StatementKind {
    fn from(value: TypeDeclaration) -> Self {
        Self::TypeDeclaration(value)
    }
}
