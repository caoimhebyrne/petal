use crate::ast::{
    statement::{
        Statement,
        StatementKind,
    },
    r#type::Type,
};

/// A function declaration within the AST.
#[derive(Debug, Clone, PartialEq)]
pub struct FunctionDeclaration {
    /// The name of the function being declared.
    pub name: String,

    /// The body of the function.
    pub body: Vec<Statement>,

    /// The parameters of the function.
    pub parameters: Vec<FunctionParameter>,

    /// The return type of the function.
    pub return_type: Option<Type>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FunctionParameter {
    /// The name of the parameter.
    pub name: String,

    /// The type of the parameter.
    pub r#type: Type,
}

impl FunctionDeclaration {
    /// Creates a new [FunctionDeclaration].
    pub fn new(
        name: String,
        body: Vec<Statement>,
        parameters: Vec<FunctionParameter>,
        return_type: Option<Type>,
    ) -> Self {
        FunctionDeclaration { name, body, parameters, return_type }
    }
}

/// Converts a [FunctionDeclaration] to a [StatementKind].
impl From<FunctionDeclaration> for StatementKind {
    fn from(value: FunctionDeclaration) -> Self {
        Self::FunctionDeclaration(value)
    }
}
