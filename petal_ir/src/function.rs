use crate::{operation::Operation, value::ValueType};
use petal_core::core::location::Location;

/// A function is the core of execution in the intermediate representation.
#[derive(Debug, Clone)]
pub struct Function {
    /// The name defined in the source code for this function.
    pub name: String,

    /// The location that the function was defined at within the source file.
    pub location: Location,

    /// The operations contained within this function's body.
    pub body: Vec<Operation>,

    /// The local variables allocated in this function's body.
    pub locals: Vec<Local>,

    /// The data used within this function.
    pub data: Vec<Vec<u8>>,

    /// Whether the function is an external function.
    pub is_external: bool,
}

/// A local defined within a [Function]'s body.
#[derive(Debug, Clone)]
pub struct Local {
    /// The name assigned to the local variable.
    pub name: String,

    /// The value type of the local variable.
    pub value_type: ValueType,

    /// The kind of local that this is.
    pub kind: LocalKind,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum LocalKind {
    /// A variable defined within the function's scope.
    Variable,

    /// A parameter within in the function's definition.
    Parameter,
}
