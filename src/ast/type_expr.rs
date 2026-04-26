/// A user defined type in the AST.
#[derive(Debug, Clone, PartialEq)]
pub enum TypeExpr {
    /// A named type, e.g. "i32".
    Named(String),
}

impl TypeExpr {
    pub fn named(name: impl Into<String>) -> Self {
        Self::Named(name.into())
    }
}
