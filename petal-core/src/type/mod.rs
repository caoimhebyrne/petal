use enum_display::EnumDisplay;

use crate::{
    error::{Error, ErrorKind, Result},
    source_span::SourceSpan,
    string_intern::StringReference,
    r#type::pool::TypePool,
};

pub mod pool;

/// A reference to a type.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct TypeId(pub usize);

/// A reference to a structure type.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct StructureId(pub usize);

/// A reference to a type with a [SourceSpan].
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct TypeReference {
    pub id: TypeId,
    pub span: SourceSpan,
}

impl TypeReference {
    /// Creates a new [TypeReference].
    pub fn new(id: TypeId, span: SourceSpan) -> Self {
        TypeReference { id, span }
    }
}

/// Represents the different kinds of types that exist.
#[derive(Debug, Clone, Eq, PartialEq, EnumDisplay)]
pub enum Type {
    /// A type that has not yet been resolved by the type checker.
    #[display("unresolved")]
    Unresolved(StringReference),

    /// A type that has been resolved.
    #[display("{0}")]
    Resolved(ResolvedType),
}

/// Represents the different kinds of fully-resolved types that exist.
#[derive(Debug, Clone, Eq, PartialEq, EnumDisplay)]
pub enum ResolvedType {
    /// An unsigned integer of a certain width.
    #[display("u{0}")]
    UnsignedInteger(u32),

    /// A signed integer of a certain width.
    #[display("i{0}")]
    SignedInteger(u32),

    /// The `void` type (empty).
    #[display("void")]
    Void,

    /// The boolean type.
    Boolean,

    /// The type used for a variadic argument receiver.
    #[display("variadic")]
    Variadic,

    /// A reference of another type. This other type may not be resolved.
    #[display("reference({0:?})")]
    Reference(TypeId),

    /// A structure type.
    Structure(StructureId),
}

/// Represents a structure defined by the user.
#[derive(Debug, Copy, Clone)]
pub struct Structure {
    /// The name of this structure.
    pub name: StringReference,
}

impl Structure {
    /// Instantiates a new [Structure].
    pub fn new(name: StringReference) -> Self {
        Structure { name }
    }
}

impl ResolvedType {
    /// Returns whether this type can be assigned to another type.
    pub fn is_assignable_to(&self, type_pool: &TypePool, other: &ResolvedType, span: SourceSpan) -> Result<bool> {
        if self == other {
            return Ok(true);
        }

        // If the other type is a reference type, the other type must be what I am referencing.
        if let ResolvedType::Reference(referenced_type_id) = other {
            return match type_pool.get_type_or_err(referenced_type_id, span)? {
                Type::Resolved(resolved) => Ok(self == resolved),
                _ => return ResolvedTypeError::unresolved_type(*referenced_type_id, span).into(),
            };
        }

        Ok(false)
    }
}

#[derive(Debug, Copy, Clone, PartialEq, EnumDisplay)]
pub enum ResolvedTypeError {
    /// The provided [TypeId] does not exist.
    #[display("The type {0:?} was expected to be resolved, but it was not")]
    UnresolvedType(TypeId),
}

impl ResolvedTypeError {
    /// Creates a new [Error] with the [ResolvedTypeError::UndefinedType] kind.
    pub fn unresolved_type(type_id: TypeId, span: SourceSpan) -> Error {
        Error::new(ResolvedTypeError::UnresolvedType(type_id), span)
    }
}

impl ErrorKind for ResolvedTypeError {}
