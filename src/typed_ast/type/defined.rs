use crate::{
    ast::statement::function_declaration::DeclarationModifier,
    core::span::Span,
    typed_ast::{
        GenericInformation,
        r#type::db::TypeId,
    },
};

#[derive(Debug, Clone, PartialEq)]
pub struct DefinedType {
    /// The modifiers applied to the type definition.
    pub modifiers: Vec<DeclarationModifier>,

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

impl Structure {
    /// Attempts to find a [`StructureField`] on this [`Structure`] which has the provided [`name`].
    ///
    /// This returns a tuple of the structure field's index to its [`TypeId`]. The [`StructureField`] itself is not
    /// returned, as the name is already known.
    pub fn find_field_by_name(&self, name: &str) -> Option<(usize, TypeId)> {
        self.fields
            .iter()
            .enumerate()
            .find(|(_, field)| field.name == name)
            .map(|(index, field)| (index, field.type_id))
    }
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
