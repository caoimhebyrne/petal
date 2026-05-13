use crate::ast::{
    statement::{
        StatementKind,
        function_declaration::DeclarationModifier,
    },
    type_expr::{
        GenericTypeParameter,
        TypeExpr,
    },
};

#[derive(Debug, Clone, PartialEq)]
pub struct TypeDeclaration {
    /// The name of the type being declared.
    pub name: String,

    /// The type being declared.
    pub type_expr: TypeExpr,

    /// The modifiers of this declaration.
    pub modifiers: Vec<DeclarationModifier>,

    /// The generic type parameters of this type.
    pub generic_type_parameters: Vec<GenericTypeParameter>,
}

impl TypeDeclaration {
    /// Creates a new [`TypeDeclaration`].
    pub fn new(
        name: String,
        type_expr: TypeExpr,
        modifiers: Vec<DeclarationModifier>,
        generic_type_parameters: Vec<GenericTypeParameter>,
    ) -> Self {
        Self { name, type_expr, modifiers, generic_type_parameters }
    }
}

impl From<TypeDeclaration> for StatementKind {
    fn from(value: TypeDeclaration) -> Self {
        Self::TypeDeclaration(value)
    }
}
