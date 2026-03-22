use crate::ast::statement::StatementKind;

/// A function declaration within the AST.
#[derive(Debug, Clone, PartialEq)]
pub struct FunctionDeclaration {
    /// The name of the function being declared.
    pub name: String,
}

impl FunctionDeclaration {
    /// Creates a new [FunctionDeclaration].
    pub fn new(name: String) -> Self {
        FunctionDeclaration { name }
    }
}

/// Converts a [FunctionDeclaration] to a [StatementKind].
impl From<FunctionDeclaration> for StatementKind {
    fn from(value: FunctionDeclaration) -> Self {
        Self::FunctionDeclaration(value)
    }
}
