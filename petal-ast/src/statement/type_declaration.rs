use petal_core::string_intern::StringReference;

use crate::statement::StatementKind;

/// A type declaration statement, e.g. `type <identifier> = <declaration>;`
#[derive(Debug, Clone, PartialEq)]
pub struct TypeDeclaration {
    /// The name of the type being declared.
    pub identifier_reference: StringReference,
}

impl TypeDeclaration {
    /// Creates a new [TypeDeclaration].
    pub fn new(identifier_reference: StringReference) -> Self {
        TypeDeclaration { identifier_reference }
    }
}

/// Allows `.into()` to be called on a [TypeDeclaration] to turn it into a [StatementKind].
impl From<TypeDeclaration> for StatementKind {
    fn from(value: TypeDeclaration) -> Self {
        StatementKind::TypeDeclaration(value)
    }
}
