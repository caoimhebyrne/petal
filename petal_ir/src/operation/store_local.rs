use crate::value::Value;

/// An`` [crate::operation::OperationKind] for storing a [Value] into a local.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct StoreLocal {
    /// The index of the local to store the value into.
    pub index: usize,

    /// The value to store into the local.
    pub value: Value,
}
