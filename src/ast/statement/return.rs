use crate::ast::{
    expression::Expression,
    statement::StatementKind,
};

/// A return statement within the AST.
#[derive(Debug, Clone, PartialEq)]
pub struct Return {
    /// The value being returned.
    pub value: Option<Expression>,
}

impl Return {
    /// Creates a new [Return].
    pub fn new(value: Option<Expression>) -> Self {
        Self { value }
    }
}

/// Converts a [Return] into a [StatementKind].
impl From<Return> for StatementKind {
    fn from(value: Return) -> Self {
        Self::Return(value)
    }
}
