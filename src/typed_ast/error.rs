use std::fmt::Display;

use crate::core::{
    error::Error,
    span::Span,
};

/// An error emitted by the typechecker.
pub struct TypecheckerError {
    /// The kind of error that this is.
    pub kind: TypecheckerErrorKind,

    /// The [`Span`] within the source code that this error occurred at.
    pub span: Span,
}

impl Error for TypecheckerError {
    fn span(&self) -> Option<Span> {
        Some(self.span)
    }
}

impl Display for TypecheckerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.kind.fmt(f)
    }
}

/// The different kinds of [`TypecheckerError`]s that exist.
pub enum TypecheckerErrorKind {
    /// The number of generic type arguments provided did not equal the number of generic type parameters.
    GenericTypeArgumentCountMismatch { expected: usize, got: usize },

    /// A type was referenced by name, but a matching type could not be resolved.
    UndeclaredTypeName(String),

    /// An identifier was encountered, but a value could not be resolved from it.
    UnresolvableIdentifierReference(String),
}

impl TypecheckerErrorKind {
    /// Creates a new [`TypecheckerError`] from this [`TypecheckerErrorKind`] using the provided [`Span`].
    pub fn at(self, span: Span) -> TypecheckerError {
        TypecheckerError { kind: self, span }
    }
}

impl Display for TypecheckerErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::GenericTypeArgumentCountMismatch { expected, got } => write!(
                f,
                "Expected {} type argument{} but got {} argument{}",
                expected,
                if *expected == 0 { "" } else { "s" },
                got,
                if *got == 0 { "" } else { "s" }
            ),

            Self::UndeclaredTypeName(name) => write!(f, "Cannot find type named '{name}'"),

            Self::UnresolvableIdentifierReference(identifier) => {
                write!(f, "Could not resolve a value for identifier '{identifier}'")
            }
        }
    }
}
