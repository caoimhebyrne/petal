use petal_core::string_intern::StringReference;

use crate::{
    statement::{Statement, StatementKind},
    r#type::Type,
};

/// A function declaration statement, e.g. `func <name>() { <body> }`
#[derive(Debug, Clone, PartialEq)]
pub struct FunctionDeclaration {
    /// The name of the function.
    pub name_reference: StringReference,

    /// The return type of the function.
    pub return_type: Type,

    /// The body of the function.
    pub body: Vec<Statement>,
}

impl FunctionDeclaration {
    /// Creates a new [FunctionDeclaration] with a [name_reference] and [body]
    pub fn new(name_reference: StringReference, return_type: Type, body: Vec<Statement>) -> Self {
        FunctionDeclaration {
            name_reference,
            return_type,
            body,
        }
    }
}

/// Allows `.into()` to be called on a [FunctionDeclaration] to turn it into a [StatementKind].
impl From<FunctionDeclaration> for StatementKind {
    fn from(value: FunctionDeclaration) -> Self {
        StatementKind::FunctionDeclaration(value)
    }
}
