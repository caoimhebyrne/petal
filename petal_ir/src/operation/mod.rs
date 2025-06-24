use crate::operation::store_local::StoreLocal;

pub mod store_local;

/// Represents an operation in the intermediate representation.
///
/// An operation does not directly map to a single instruction in the code generator
/// or interpreter.
///
/// See [OperationKind].
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Operation {
    /// The kind of operation that this represents.
    pub kind: OperationKind,
}

/// The different kinds of operations in the intermediate representation.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum OperationKind {
    /// Stores a [crate::value::Value] into a local at the provided index.
    StoreLocal(StoreLocal),
}
