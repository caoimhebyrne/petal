use crate::{source_span::SourceSpan, string_intern::StringReference};

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
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Type {
    /// A type that has not yet been resolved by the type checker.
    Unresolved(StringReference),

    /// A type that has been resolved.
    Resolved(ResolvedType),

    /// A type which is a reference of another type (e.g. `&i32`).
    Reference(TypeId),
}

/// Represents the different kinds of fully-resolved types that exist.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum ResolvedType {
    /// An integer of a certain width.
    Integer(u8),

    /// The `void` type (empty).
    Void,
}
