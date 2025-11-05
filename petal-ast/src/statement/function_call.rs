use petal_core::string_intern::StringReference;

use crate::{expression::Expression, statement::StatementKind};

/// A call to a function. This is not only a [crate::Statement], it is also a [crate::Expression].
#[derive(Debug, Clone, PartialEq)]
pub struct FunctionCall {
    /// The name of the function being called.
    pub name_reference: StringReference,

    /// The arguments being passed as parameters to the function.
    pub arguments: Vec<Expression>,
}

impl FunctionCall {
    pub fn new(name_reference: StringReference, arguments: Vec<Expression>) -> Self {
        FunctionCall {
            name_reference,
            arguments,
        }
    }
}

/// Allows `.into()` to be called on a [ReturnStatement] to turn it into a [StatementKind].
impl From<FunctionCall> for StatementKind {
    fn from(value: FunctionCall) -> Self {
        StatementKind::FunctionCall(value)
    }
}
