use crate::value::Value;

/// A call to a function.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FunctionCall {
    /// The name of the function being called.
    pub name: String,

    /// The arguments being passed to the function.
    pub arguments: Vec<Value>,
}
