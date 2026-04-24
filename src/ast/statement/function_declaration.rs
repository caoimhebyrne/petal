use crate::ast::statement::{
    Statement,
    StatementKind,
};

/// A function declaration within the AST.
#[derive(Debug, Clone, PartialEq)]
pub struct FunctionDeclaration {
    /// The name of the function being declared.
    pub name: String,

    /// The body of the function.
    pub body: Vec<Statement>,
}

impl FunctionDeclaration {
    /// Creates a new [FunctionDeclaration].
    pub fn new(name: String, body: Vec<Statement>) -> Self {
        FunctionDeclaration { name, body }
    }
}

/// Converts a [FunctionDeclaration] to a [StatementKind].
impl From<FunctionDeclaration> for StatementKind {
    fn from(value: FunctionDeclaration) -> Self {
        Self::FunctionDeclaration(value)
    }
}
