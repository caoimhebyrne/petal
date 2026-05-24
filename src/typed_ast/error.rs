use std::fmt::Display;

use crate::{
    core::{
        error::Error,
        span::Span,
    },
    typed_ast::r#type::db::{
        TypeDb,
        TypeId,
    },
};

/// An error emitted by the typechecker.
#[derive(Debug)]
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
#[derive(Debug)]
pub enum TypecheckerErrorKind {
    /// An ambiguous function call was encountered, multiple definitions qualified.
    AmbiguousFunctionCall(usize),

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

    /// A function call expression was encountered, but the target of the call was not valid.
    InvalidFunctionCallTarget,

    /// A field was not provided in a structure initialization expression.
    MissingStructureFieldInInitializer(String),

    /// The number of field initializers provided did not equal the number of fields on the structure type.
    StructureInitializationFieldCountMismatch { expected: usize, got: usize },

    /// A type was expected, but an unsupported type was received.
    TypeMismatch { expected: String, got: String },

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

    /// Creates a new [`TypecheckerErrorKind::TypeMismatch`] from the provided [`TypeDb`] and [`TypeId`]s.
    pub fn type_mismatch(type_db: &TypeDb, expected_type_id: TypeId, got_type_id: TypeId) -> TypecheckerErrorKind {
        TypecheckerErrorKind::TypeMismatch {
            expected: type_db.get_type_description(expected_type_id),
            got: type_db.get_type_description(got_type_id),
        }
    }
}

impl Display for TypecheckerErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::AmbiguousFunctionCall(candidates) => {
                write!(f, "This function call is ambiguous, there are {} possible candidates", candidates)
            }

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

            Self::InvalidFunctionCallTarget => {
                write!(
                    f,
                    "The target of this function call is invalid (expected a function name, value reference, or type name)"
                )
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

            Self::TypeMismatch { expected, got } => {
                write!(f, "Expected a value of type '{expected}', but received a value of type '{got}'")
            }

            Self::UndeclaredFunction(name) => write!(f, "Cannot find function named '{name}'"),

            Self::UndeclaredTypeName(name) => write!(f, "Cannot find type named '{name}'"),

            Self::UnresolvableIdentifierReference(identifier) => {
                write!(f, "Could not resolve a value for identifier '{identifier}'")
            }
        }
    }
}
