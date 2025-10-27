use petal_core::string_intern::StringReference;

use crate::{expression::Expression, statement::StatementKind, r#type::Type};

/// A variable declaration statement, e.g: `<type> <identifier> = <expression>;`
#[derive(Debug, Clone, PartialEq)]
pub struct VariableDeclaration {
    /// The name of the variable being declared.
    pub identifier_reference: StringReference,

    /// The type of the variable being declared.
    pub r#type: Type,

    /// The value being assigned to the variable.
    pub value: Expression,
}

impl VariableDeclaration {
    /// Creates a new [VariableDeclaration] with a [name] and [value].
    pub fn new(identifier_reference: StringReference, r#type: Type, value: Expression) -> Self {
        VariableDeclaration {
            identifier_reference,
            r#type,
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
