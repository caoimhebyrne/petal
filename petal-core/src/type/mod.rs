use crate::{source_span::SourceSpan, string_intern::StringReference, r#type::pool::TypeId};

pub mod pool;

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Type {
    /// The kind of type that this is.
    pub kind: TypeKind,

    /// The span within the source code that this type occurred at.
    pub span: SourceSpan,
}

/// Represents the different kinds of types that exist.
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum TypeKind {
    /// A type that has not yet been resolved by the type checker.
    Unresolved(StringReference),

    /// A type that has been resolved.
    Resolved(ResolvedTypeKind),

    /// A type which is a reference of another type (e.g. `&i32`).
    Reference(TypeId),
}

/// Represents the different kinds of fully-resolved types that exist.
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum ResolvedTypeKind {
    /// An integer of a certain width.
    Integer(u8),

    /// The `void` type (empty).
    Void,
}
