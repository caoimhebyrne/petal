use std::fmt::Display;

use crate::{
    core::{
        error::Error,
        span::Span,
    },
    typechecker::r#type::Type,
};

/// An AST error.
#[derive(Debug, PartialEq)]
pub struct TypecheckerError {
    /// The kind of typechecker error that this is.
    pub kind: TypecheckerErrorKind,

    /// The [`Span`] that the error occurred at.
    pub span: Span,
}

/// The different kinds of [`TypecheckerError`]s that exist.
#[derive(Debug, PartialEq)]
pub enum TypecheckerErrorKind {
    BinaryOperationNotSupported(Type),
    IncompatibleBinaryOperationTypes { left: Type, right: Type },
    IncompatibleVariableDeclarationTypes { declared: Type, value: Type },
    IncompatibleReturnTypes { declared: Type, value: Type },
    InvalidFunctionCall { name: String, parameters: Vec<Type>, arguments: Vec<Type> },
    UndeclaredFunction(String),
    UndeclaredVariable(String),
    UnknownType(String),
}

impl TypecheckerErrorKind {
    /// Returns an [TypecheckerError] from this [TypecheckerErrorKind] at the provided [Span].
    pub fn at(self, span: Span) -> TypecheckerError {
        TypecheckerError { kind: self, span }
    }
}

impl TypecheckerError {
    /// Creates a new [`TypecheckerError`].
    pub fn new(kind: TypecheckerErrorKind, span: Span) -> Self {
        TypecheckerError { kind, span }
    }
}

impl Display for TypecheckerErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::BinaryOperationNotSupported(r#type) => {
                write!(f, "Binary operations are not supported on the type '{:?}'", r#type)
            }

            Self::UnknownType(name) => write!(f, "Unknown type: '{name}'"),

            Self::IncompatibleBinaryOperationTypes { left, right } => {
                write!(f, "Binary operation has two values of incompatible types: '{:?}' and '{:?}'", left, right)
            }

            Self::IncompatibleVariableDeclarationTypes { declared, value } => {
                write!(f, "Unable to assign value of type '{:?}' to variable of type '{:?}'", value, declared)
            }

            Self::IncompatibleReturnTypes { declared, value } => {
                write!(
                    f,
                    "Unable to return value of type '{:?}' from function with return type '{:?}'",
                    value, declared
                )
            }

            Self::UndeclaredFunction(name) => write!(f, "Function '{name}' has not been declared yet"),

            Self::UndeclaredVariable(name) => write!(f, "Variable '{name}' has not been declared yet"),

            Self::InvalidFunctionCall { name, parameters, arguments } => write!(
                f,
                "Function '{}' has parameters of types {:?}, but function call has arguments of types {:?}",
                name, parameters, arguments
            ),
        }
    }
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
