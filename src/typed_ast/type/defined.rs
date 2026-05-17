use crate::{
    core::span::Span,
    typed_ast::{
        GenericInformation,
        r#type::db::TypeId,
    },
};

#[derive(Debug, Clone, PartialEq)]
pub struct DefinedType {
    /// The name of the defined type.
    pub name: String,

    /// The kind of type that was defined.
    pub kind: DefinedTypeKind,

    /// Information about the generic types within this type, this is typically populated during the
    /// generation of the specialized type, and may be read by later stages.
    pub generic_information: Option<GenericInformation>,

    /// The location in the source code that this type was defined at.
    pub span: Span,
}

/// The different kinds of [`DefinedType`]s that exist.
#[derive(Debug, Clone, PartialEq)]
pub enum DefinedTypeKind {
    /// A structure.
    Structure(Structure),
}

/// A structure defined within a program by a user.
#[derive(Debug, Clone, PartialEq)]
pub struct Structure {
    /// The fields of the structure.
    pub fields: Vec<StructureField>,
}

/// A field on a structure.
#[derive(Debug, Clone, PartialEq)]
pub struct StructureField {
    /// The name of the field.
    pub name: String,

    /// The type of the field.
    pub type_id: TypeId,

    /// The position in the source code that the field was defined.
    pub span: Span,
}
