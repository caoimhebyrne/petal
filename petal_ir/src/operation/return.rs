use crate::value::Value;

/// An [crate::operation::OperationKind] for returning a [Value] from a local.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Return {
    /// The value being returned.
    pub value: Option<Value>,
}
