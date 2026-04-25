/// A user defined type in the AST.
#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    /// A named type, e.g. "i32".
    Named(String),
}
