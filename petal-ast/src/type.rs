use std::fmt::Display;

use petal_core::{source_span::SourceSpan, string_intern::StringReference};

/// A type associated with an AST node.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Type {
    /// The kind of type that this is.
    pub kind: TypeKind,

    /// The span within the source code that the type was defined at.
    pub span: SourceSpan,
}

impl Type {
    /// Creates a new [Type] with the provided [TypeKind] and [SourceSpan].
    pub fn new(kind: impl Into<TypeKind>, span: SourceSpan) -> Self {
        Type {
            kind: kind.into(),
            span,
        }
    }

    /// Creates an unresolved type.
    pub fn unresolved(name: StringReference, span: SourceSpan) -> Self {
        Type::new(TypeKind::Unresolved(name), span)
    }

    /// Creates a void type.
    pub fn void(span: SourceSpan) -> Self {
        Type::new(ResolvedTypeKind::Void, span)
    }
}

/// Represents the kinds of types that can be associated with an AST node.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TypeKind {
    /// The type is unresolved.
    Unresolved(StringReference),

    /// The type has been resolved.
    Resolved(ResolvedTypeKind),
}

impl Display for TypeKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TypeKind::Unresolved(_) => write!(f, "<unresolved>"),
            TypeKind::Resolved(kind) => write!(f, "{}", kind),
        }
    }
}

/// The kinds of resolved types that exist.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResolvedTypeKind {
    /// An integer type.
    Integer(u32),

    /// The 'void' type.
    Void,
}

impl Display for ResolvedTypeKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ResolvedTypeKind::Integer(size) => write!(f, "i{}", size),
            ResolvedTypeKind::Void => write!(f, "void"),
        }
    }
}

impl Into<TypeKind> for ResolvedTypeKind {
    fn into(self) -> TypeKind {
        TypeKind::Resolved(self)
    }
}
