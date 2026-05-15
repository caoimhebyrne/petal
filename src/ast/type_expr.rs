use crate::{
    core::span::Span,
    typechecker::r#type::Type,
};

/// A user defined type in the AST.
#[derive(Debug, Clone, PartialEq)]
pub enum TypeExpr {
    /// A named type, e.g. "i32".
    Named {
        /// The name of the type.
        name: String,

        /// The generic arguments passed _to_ the type.
        generic_type_arguments: Vec<GenericTypeArgument>,
    },

    /// A reference to another type.
    Reference(Box<TypeExpr>),

    /// An optional wrapping a type.
    Optional(Box<TypeExpr>),

    /// A structure definition.
    Structure { fields: Vec<StructureField> },

    /// An enumeration definition.
    Enum { variants: Vec<EnumVariant> },
}

impl TypeExpr {
    pub fn named(name: impl Into<String>) -> Self {
        Self::Named { name: name.into(), generic_type_arguments: vec![] }
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

/// An enumeration variant.
#[derive(Debug, Clone)]
pub struct EnumVariant {
    /// The name of the variant.
    pub name: String,

    /// The span that this variant was defined at in the source code.
    pub span: Span,
}

impl PartialEq for EnumVariant {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

/// A single generic argument to a generic function or  generic type.
#[derive(Debug, Clone)]
pub struct GenericTypeArgument {
    /// The expression of the argument.
    pub type_expr: TypeExpr,

    /// The resolved type of the argument.
    pub r#type: Type,

    /// The span within the source code that the parameter was defined at.
    pub span: Span,
}

impl GenericTypeArgument {
    /// Creates a new [`GenericTypeArgument`].
    pub fn new(type_expr: TypeExpr, span: Span) -> Self {
        Self { type_expr, r#type: Type::default(), span }
    }
}

impl PartialEq for GenericTypeArgument {
    fn eq(&self, other: &Self) -> bool {
        self.type_expr == other.type_expr && self.r#type == other.r#type
    }
}

/// A single generic parameter to a function or type.
///
/// TODO: Constraints
#[derive(Debug, Clone, PartialEq)]
pub struct GenericTypeParameter {
    /// The name of the parameter.
    pub name: String,

    /// The span within the source code that the parameter was defined at.
    pub span: Span,
}

impl GenericTypeParameter {
    /// Creates a new [`GenericTypeParameter`].
    pub fn new(name: String, span: Span) -> Self {
        Self { name, span }
    }
}
