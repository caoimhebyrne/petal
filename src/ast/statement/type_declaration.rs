use crate::ast::{
    statement::{
        StatementKind,
        function_declaration::DeclarationModifier,
    },
    type_expr::TypeExpr,
};

#[derive(Debug, Clone, PartialEq)]
pub struct TypeDeclaration {
    /// The name of the type being declared.
    pub name: String,

    /// The type being declared.
    pub type_expr: TypeExpr,

    /// The modifiers of this declaration.
    pub modifiers: Vec<DeclarationModifier>,
}

impl TypeDeclaration {
    /// Creates a new [`TypeDeclaration`].
    pub fn new(name: String, type_expr: TypeExpr, modifiers: Vec<DeclarationModifier>) -> Self {
        Self { name, type_expr, modifiers }
    }
}

impl From<TypeDeclaration> for StatementKind {
    fn from(value: TypeDeclaration) -> Self {
        Self::TypeDeclaration(value)
    }
}
