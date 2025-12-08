use std::collections::HashMap;

use enum_display::EnumDisplay;

use crate::{
    error::{Error, ErrorKind, Result},
    source_span::SourceSpan,
    r#type::{Type, TypeId},
};

/// A type pool stores references to [Type]s that can later be resolved.
///
/// The alternative to this is storing the [Type] struct everywhere, but that becmes complicated when you have nested
/// types. Keeping the `Copy` is important here.
pub struct TypePool {
    /// A vector of types that have been allocated.
    types: HashMap<TypeId, Type>,
}

impl TypePool {
    /// Constructs a new [TypePool] instance.
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        TypePool { types: HashMap::new() }
    }

    /// Allocates a [TypeId] for the provided [Type].
    pub fn allocate(&mut self, r#type: Type) -> TypeId {
        // If a the same type exists, then we can return its type ID.
        if let Some(type_id) = self
            .types
            .iter()
            .filter_map(|(key, value)| if *value == r#type { Some(key) } else { None })
            .nth(0)
        {
            return *type_id;
        }

        let type_id = TypeId(self.types.len());

        self.types.insert(type_id, r#type);

        type_id
    }

    /// Returns the [Type] associated with the provided [TypeId], if it exists.
    pub fn get_type(&self, id: &TypeId) -> Option<&Type> {
        self.types.get(id)
    }

    /// Returns a mutable reference to the [Type] associated with the proivded [TypeId], if it exists.
    pub fn get_type_mut(&mut self, id: &TypeId) -> Option<&mut Type> {
        self.types.get_mut(id)
    }

    /// Returns the [Type] associated with the provided [TypeId].
    ///
    /// Errors:
    /// - [TypePoolError::UndefinedType] If a type has not been defined for the provided [TypeId].
    pub fn get_type_or_err(&self, id: &TypeId, span: SourceSpan) -> Result<&Type> {
        self.get_type(id).ok_or(TypePoolError::undefined_type(*id, span))
    }

    /// Returns a mutable reference to the [Type] associated with the provided [TypeId].
    ///
    /// Errors:
    /// - [TypePoolError::UndefinedType] If a type has not been defined for the provided [TypeId].
    pub fn get_type_mut_or_err(&mut self, id: &TypeId, span: SourceSpan) -> Result<&mut Type> {
        self.get_type_mut(id).ok_or(TypePoolError::undefined_type(*id, span))
    }
}

#[derive(Debug, Copy, Clone, PartialEq, EnumDisplay)]
pub enum TypePoolError {
    /// The provided [TypeId] does not exist.
    #[display("The type of ID '{0:?}' has not been defined yet")]
    UndefinedType(TypeId),
}

impl TypePoolError {
    /// Creates a new [Error] with the [TypePoolError::UndefinedType] kind.
    pub fn undefined_type(id: TypeId, span: SourceSpan) -> Error {
        Error::new(TypePoolError::UndefinedType(id), span)
    }
}

impl ErrorKind for TypePoolError {}
