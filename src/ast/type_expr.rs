use crate::{
    core::span::Span,
    typechecker::r#type::Type,
};

/// A user defined type in the AST.
#[derive(Debug, Clone, PartialEq)]
pub enum TypeExpr {
    /// A named type, e.g. "i32".
    Named(String),

    /// A reference to another type.
    Reference(Box<TypeExpr>),

    /// An optional wrapping a type.
    Optional(Box<TypeExpr>),

    /// A structure definition.
    Structure { fields: Vec<StructureField> },
}

impl TypeExpr {
    pub fn named(name: impl Into<String>) -> Self {
        Self::Named(name.into())
    }

    pub fn reference(type_expr: TypeExpr) -> Self {
        Self::Reference(type_expr.into())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StructureField {
    /// The name of the field.
    pub name: String,

    /// The declared type of the field.
    pub type_expr: TypeExpr,

    /// The resolved type of the field.
    pub r#type: Type,

    /// The span that this field was defined at in the source code.
    pub span: Span,
}

impl StructureField {
    /// Creates a new [`StructureField`].
    pub fn new(name: String, type_expr: TypeExpr, span: Span) -> Self {
        Self { name, type_expr, r#type: Type::Unknown, span }
    }
}
