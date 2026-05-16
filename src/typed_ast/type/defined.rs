#[derive(Debug, Clone, PartialEq)]
pub struct DefinedType {
    /// The name of the defined type.
    pub name: String,

    /// The kind of type that was defined.
    pub kind: DefinedTypeKind,
}

/// The different kinds of [`DefinedType`]s that exist.
#[derive(Debug, Clone, PartialEq)]
pub enum DefinedTypeKind {
    /// A structure.
    Structure(Structure),
}

/// A structure defined within a program by a user.
#[derive(Debug, Clone, PartialEq)]
pub struct Structure;
