use petal_core::string_intern::StringReference;

use crate::statement::StatementKind;

/// A type declaration statement, e.g. `type <identifier> = <declaration>;`
#[derive(Debug, Clone, PartialEq)]
pub struct TypeDeclaration {
    /// The name of the type being declared.
    pub identifier_reference: StringReference,

    /// The kind of type being declared.
    pub kind: TypeDeclarationKind,
}

impl TypeDeclaration {
    /// Creates a new [TypeDeclaration].
    pub fn new(identifier_reference: StringReference, kind: TypeDeclarationKind) -> Self {
        TypeDeclaration {
            identifier_reference,
            kind: kind.into(),
        }
    }
}

/// Allows `.into()` to be called on a [TypeDeclaration] to turn it into a [StatementKind].
impl From<TypeDeclaration> for StatementKind {
    fn from(value: TypeDeclaration) -> Self {
        StatementKind::TypeDeclaration(value)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum TypeDeclarationKind {
    /// A structure declaration.
    Structure(StructureDeclaration),
}

#[derive(Debug, Clone, PartialEq)]
pub struct StructureDeclaration {}

impl StructureDeclaration {
    pub fn new() -> Self {
        StructureDeclaration {}
    }
}

/// Allows `.into()` to be called on a [StructureDeclaration] to turn it into a [TypeDeclarationKind].
impl From<StructureDeclaration> for TypeDeclarationKind {
    fn from(value: StructureDeclaration) -> Self {
        TypeDeclarationKind::Structure(value)
    }
}
