use petal_core::string_intern::StringReference;

use crate::{expression::Expression, statement::StatementKind};

/// A variable assignment statement, e.g: ` <identifier> = <expression>;`
#[derive(Debug, Clone, PartialEq)]
pub struct VariableAssignment {
    /// The name of the variable that is having a value assigned to it.
    pub identifier_reference: StringReference,

    /// The value being assigned to the variable.
    pub value: Expression,
}

impl VariableAssignment {
    /// Creates a new [VariableDeclaration] with a [name] and [value].
    pub fn new(identifier_reference: StringReference, value: Expression) -> Self {
        VariableAssignment {
            identifier_reference,
            value,
        }
    }
}

/// Allows `.into()` to be called on a [VariableAssignment] to turn it into a [StatementKind].
impl From<VariableAssignment> for StatementKind {
    fn from(value: VariableAssignment) -> Self {
        StatementKind::VariableAssignment(value)
    }
}
