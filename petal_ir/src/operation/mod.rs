use crate::{
    operation::{r#return::Return, store_local::StoreLocal},
    value::{Value, function_call::FunctionCall},
};

pub mod r#return;
pub mod store_local;

/// Represents an operation in the intermediate representation.
///
/// An operation does not directly map to a single instruction in the code generator
/// or interpreter.
///
/// See [OperationKind].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Operation {
    /// The kind of operation that this represents.
    pub kind: OperationKind,
}

/// The different kinds of operations in the intermediate representation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OperationKind {
    /// Stores a [crate::value::Value] into a local at the provided index.
    StoreLocal(StoreLocal),

    /// Returns a [crate::value::Value] from a function.
    Return(Return),

    /// Calls a function without a result.
    FunctionCall(FunctionCall),
}

impl Operation {
    pub fn new(kind: OperationKind) -> Operation {
        Operation { kind }
    }

    pub fn new_store_local(index: usize, value: Value) -> Operation {
        Operation {
            kind: OperationKind::StoreLocal(StoreLocal { index, value }),
        }
    }

    pub fn new_return(value: Option<Value>) -> Operation {
        Operation {
            kind: OperationKind::Return(Return { value }),
        }
    }

    pub fn new_function_call(name: String, arguments: Vec<Value>) -> Operation {
        Operation {
            kind: OperationKind::FunctionCall(FunctionCall { name, arguments }),
        }
    }
}
