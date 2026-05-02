/// A user defined type in the AST.
#[derive(Debug, Clone, PartialEq)]
pub enum TypeExpr {
    /// A named type, e.g. "i32".
    Named(String),

    /// A reference to another type.
    Reference(Box<TypeExpr>),
}

impl TypeExpr {
    pub fn named(name: impl Into<String>) -> Self {
        Self::Named(name.into())
    }

    pub fn reference(type_expr: TypeExpr) -> Self {
        Self::Reference(type_expr.into())
    }
}
