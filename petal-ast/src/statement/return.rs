use crate::{expression::Expression, statement::StatementKind};

/// A return statement, e.g. `return <value>;`
#[derive(Debug, Clone, PartialEq)]
pub struct ReturnStatement {
    /// The value being returned.
    pub value: Option<Expression>,
}

impl ReturnStatement {
    /// Creates a new [ReturnStatement] with a [value].
    pub fn new(value: Option<Expression>) -> Self {
        ReturnStatement { value }
    }
}

/// Allows `.into()` to be called on a [ReturnStatement] to turn it into a [StatementKind].
impl From<ReturnStatement> for StatementKind {
    fn from(value: ReturnStatement) -> Self {
        StatementKind::ReturnStatement(value)
    }
}
