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

    /// Converts this type to a resolved type of the provided [ResolvedTypeKind].
    pub fn resolve(&mut self, kind: ResolvedTypeKind) {
        self.kind = TypeKind::Resolved(kind);
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

/// The kinds of resolved types that exist.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResolvedTypeKind {
    /// An integer type.
    Integer(usize),

    /// The 'void' type.
    Void,
}

impl Into<TypeKind> for ResolvedTypeKind {
    fn into(self) -> TypeKind {
        TypeKind::Resolved(self)
    }
}
