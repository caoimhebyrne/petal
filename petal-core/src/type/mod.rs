use enum_display::EnumDisplay;

use crate::{error::Result, source_span::SourceSpan, string_intern::StringReference, r#type::pool::TypePool};

pub mod pool;

/// A reference to a type.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct TypeId(pub usize);

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
#[derive(Debug, Copy, Clone, Eq, PartialEq, EnumDisplay)]
pub enum Type {
    /// A type that has not yet been resolved by the type checker.
    #[display("unresolved")]
    Unresolved(StringReference),

    /// A type that has been resolved.
    #[display("{0}")]
    Resolved(ResolvedType),
}

/// Represents the different kinds of fully-resolved types that exist.
#[derive(Debug, Copy, Clone, Eq, PartialEq, EnumDisplay)]
pub enum ResolvedType {
    /// An unsigned integer of a certain width.
    #[display("i{0}")]
    UnsignedInteger(u32),

    /// A signed integer of a certain width.
    #[display("i{0}")]
    SignedInteger(u32),

    /// The `void` type (empty).
    #[display("void")]
    Void,

    /// A reference of another type. This other type may not be resolved.
    #[display("reference({0:?})")]
    Reference(TypeId),
}

impl ResolvedType {
    /// Returns whether this type can be assigned to another type.
    pub fn is_assignable_to(&self, type_pool: &TypePool, other: &ResolvedType, span: SourceSpan) -> Result<bool> {
        // If the other type is a reference type, the other type must be what I am referencing.
        if let ResolvedType::Reference(referenced_type_id) = other {
            let referenced_type = match type_pool.get_type_or_err(referenced_type_id, span)? {
                Type::Resolved(resolved) => resolved,
                _ => panic!(),
            };

            return Ok(self == referenced_type);
        }

        Ok(self == other)
    }
}
