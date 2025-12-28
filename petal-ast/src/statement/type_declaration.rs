use petal_core::string_intern::StringReference;

use crate::statement::TopLevelStatementNodeKind;

/// A type declaration.
#[derive(Debug, PartialEq, Clone)]
pub struct TypeDeclaration {
    /// The name of the type being declared.
    pub name: StringReference,

    /// The kind of type declaration that this is.
    pub kind: TypeDeclarationKind,
}

impl TypeDeclaration {
    /// Instantiates a [TypeDeclaration].
    pub fn new(name: StringReference, kind: TypeDeclarationKind) -> Self {
        TypeDeclaration { name, kind }
    }
}

/// A kind of type declaration.
#[derive(Debug, PartialEq, Clone)]
pub enum TypeDeclarationKind {
    /// A structure type declaration.
    Structure(StructureTypeDeclaration),
}

/// A structure type declaration.
#[derive(Debug, PartialEq, Clone)]
pub struct StructureTypeDeclaration {}

impl StructureTypeDeclaration {
    /// Instantiates a [StructureTypeDeclaration]
    pub fn new() -> Self {
        StructureTypeDeclaration {}
    }
}

impl From<TypeDeclaration> for TopLevelStatementNodeKind {
    fn from(value: TypeDeclaration) -> Self {
        TopLevelStatementNodeKind::TypeDeclaration(value)
    }
}

impl From<StructureTypeDeclaration> for TypeDeclarationKind {
    fn from(value: StructureTypeDeclaration) -> Self {
        TypeDeclarationKind::Structure(value)
    }
}
