use crate::function::Local;

/// A scope for a function being generated in the intermediate representation.
#[derive(Debug, Clone)]
pub struct FunctionScope {
    /// The locals defined in this function's scope.
    pub locals: Vec<Local>,

    /// The byte data used within this function.
    pub data: Vec<Vec<u8>>,
}

impl FunctionScope {
    /// Creates a new empty function scope.
    pub fn new() -> FunctionScope {
        FunctionScope {
            locals: vec![],
            data: vec![],
        }
    }
}
