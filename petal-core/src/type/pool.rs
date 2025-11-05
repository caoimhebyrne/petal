use enum_display::EnumDisplay;

use crate::{
    error::{Error, ErrorKind, Result},
    source_span::SourceSpan,
    r#type::Type,
};

/// A reference to a type.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct TypeId(usize);

/// A type pool stores references to [Type]s that can later be resolved.
///
/// The alternative to this is storing the [Type] struct everywhere, but that becmes complicated when you have nested
/// types. Keeping the `Copy` is important here.
///
/// This is a naieve implementation. If [TypePool::allocate] is called more than once with the same [Type] value it will
/// always have a new [TypeId] allocated for it.
pub struct TypePool {
    /// A vector of types that have been allocated.
    types: Vec<Type>,
}

impl TypePool {
    /// Constructs a new [TypePool] instance.
    pub fn new() -> Self {
        TypePool { types: Vec::new() }
    }

    /// Allocates a [TypeId] for the provided [Type].
    pub fn allocate(&mut self, r#type: Type) -> TypeId {
        let type_id = TypeId(self.types.len());

        self.types.push(r#type);

        type_id
    }

    /// Returns the [Type] associated with the provided [TypeId], if it exists.
    pub fn get_type(&self, id: &TypeId) -> Option<&Type> {
        self.types.get(id.0)
    }

    /// Returns the [Type] associated with the provided [TypeId].
    ///
    /// Errors:
    /// - [TypePoolError::UndefinedType] If a type has not been defined for the provided [TypeId].
    pub fn get_type_or_err(&self, id: &TypeId, span: SourceSpan) -> Result<&Type> {
        self.types.get(id.0).ok_or(TypePoolError::undefined_type(*id, span))
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
