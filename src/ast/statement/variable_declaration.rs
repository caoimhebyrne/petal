use crate::{
    ast::{expression::Expression, statement::StatementKind},
    core::string_intern::StringReference,
};

/// A variable declaration statement, e.g: `let <identifier> = <expression>;`
#[derive(Debug, Clone, PartialEq)]
pub struct VariableDeclaration {
    /// The name of the variable being declared.
    pub identifier_reference: StringReference,

    /// The value being assigned to the variable.
    pub value: Expression,
}

impl VariableDeclaration {
    /// Creates a new [VariableDeclaration] with a [name] and [value].
    pub fn new(identifier_reference: StringReference, value: Expression) -> Self {
        VariableDeclaration {
            identifier_reference,
            value,
        }
    }
}

/// Allows `.into()` to be called on a [VariableDeclaration] to turn it into a [StatementKind].
impl From<VariableDeclaration> for StatementKind {
    fn from(value: VariableDeclaration) -> Self {
        StatementKind::VariableDeclaration(value)
    }
}
