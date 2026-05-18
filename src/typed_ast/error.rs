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
    /// A type expression was provided for a type definition, but the expression was not a definition kind.
    ExpectedTypeDefinition,

    /// A structure type was expected, but another type kind was received.
    ExpectedStructureType,

    /// The number of generic type arguments provided did not equal the number of generic type parameters.
    GenericTypeArgumentCountMismatch { expected: usize, got: usize },

    /// An expression was the target of an assignment expression, but it wasn't supported.
    InvalidAssignmentTarget,

    /// A dereference expression was encountered, where the target of the expression was not a reference.
    InvalidDereferenceTarget,

    /// A field was not provided in a structure initialization expression.
    MissingStructureFieldInInitializer(String),

    /// The number of field initializers provided did not equal the number of fields on the structure type.
    StructureInitializationFieldCountMismatch { expected: usize, got: usize },

    /// A function call was made, but a matching function could not be found.
    UndeclaredFunction(String),

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
            Self::ExpectedTypeDefinition => {
                write!(f, "Expected any type definition (struct, enum), but got a plain type expression instead")
            }

            Self::ExpectedStructureType => {
                write!(f, "Expected a structure type to be the target of this expression, but got some other type")
            }

            Self::GenericTypeArgumentCountMismatch { expected, got } => write!(
                f,
                "Expected {} type argument{} but got {} argument{}",
                expected,
                if *expected == 1 { "" } else { "s" },
                got,
                if *got == 1 { "" } else { "s" }
            ),

            Self::InvalidAssignmentTarget => write!(
                f,
                "The target of this assignment expression is invalid (expected a variable name or a dereference expression)"
            ),

            Self::InvalidDereferenceTarget => {
                write!(f, "You cannot dereference this expression type, it must be a reference type")
            }

            Self::MissingStructureFieldInInitializer(name) => {
                write!(f, "A value was not provided for field '{name}' in the structure initializer")
            }

            Self::StructureInitializationFieldCountMismatch { expected, got } => write!(
                f,
                "Expected {} field initializer{} but got {} field initializer{}",
                expected,
                if *expected == 1 { "" } else { "s" },
                got,
                if *got == 1 { "" } else { "s" }
            ),

            Self::UndeclaredFunction(name) => write!(f, "Cannot find function named '{name}'"),

            Self::UndeclaredTypeName(name) => write!(f, "Cannot find type named '{name}'"),

            Self::UnresolvableIdentifierReference(identifier) => {
                write!(f, "Could not resolve a value for identifier '{identifier}'")
            }
        }
    }
}
