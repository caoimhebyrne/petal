use crate::function::Local;

/// A scope for a function being generated in the intermediate representation.
#[derive(Debug, Clone)]
pub struct FunctionScope {
    /// The locals defined in this function's scope.
    pub locals: Vec<Local>,

    /// The parameters defined in this function's scope.
    pub parameters: Vec<Local>,
}

impl FunctionScope {
    /// Creates a new empty function scope.
    pub fn new() -> FunctionScope {
        FunctionScope {
            locals: vec![],
            parameters: vec![],
        }
    }
}
