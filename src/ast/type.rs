/// A user defined type in the AST.
#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    /// A named type, e.g. "i32".
    Named(String),
}

impl Type {
    pub fn named(name: impl Into<String>) -> Self {
        Self::Named(name.into())
    }
}
