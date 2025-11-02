use petal_core::string_intern::StringReference;

use crate::statement::StatementKind;

/// A call to a function. This is not only a [crate::Statement], it is also a [crate::Expression].
#[derive(Debug, Clone, PartialEq)]
pub struct FunctionCall {
    /// The name of the function being called.
    pub name_reference: StringReference,
}

impl FunctionCall {
    pub fn new(name_reference: StringReference) -> Self {
        FunctionCall { name_reference }
    }
}

/// Allows `.into()` to be called on a [ReturnStatement] to turn it into a [StatementKind].
impl From<FunctionCall> for StatementKind {
    fn from(value: FunctionCall) -> Self {
        StatementKind::FunctionCall(value)
    }
}
