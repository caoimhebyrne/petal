use crate::core::location::Location;
use std::fmt::Display;

use super::r#type::kind::TypeKind;

#[derive(Debug, Clone)]
pub enum TypecheckerErrorKind {
    UndefinedVariable(String),
    UndefinedFunction(String),
    UnableToResolveType(String),
    ExpectedReturn,
    MismatchedType { expected: TypeKind, received: TypeKind },
}

#[derive(Debug, Clone)]
pub struct TypecheckerError {
    pub kind: TypecheckerErrorKind,
    pub location: Location,
}

impl TypecheckerError {
    pub fn undefined_variable(name: String, location: Location) -> TypecheckerError {
        TypecheckerError {
            kind: TypecheckerErrorKind::UndefinedVariable(name),
            location,
        }
    }

    pub fn undefined_function(name: String, location: Location) -> TypecheckerError {
        TypecheckerError {
            kind: TypecheckerErrorKind::UndefinedFunction(name),
            location,
        }
    }

    pub fn unable_to_resolve_type(name: String, location: Location) -> TypecheckerError {
        TypecheckerError {
            kind: TypecheckerErrorKind::UnableToResolveType(name),
            location,
        }
    }

    pub fn mismatched_type(expected: TypeKind, received: TypeKind, location: Location) -> TypecheckerError {
        TypecheckerError {
            kind: TypecheckerErrorKind::MismatchedType { expected, received },
            location,
        }
    }

    pub fn expected_return(location: Location) -> TypecheckerError {
        TypecheckerError {
            kind: TypecheckerErrorKind::ExpectedReturn,
            location,
        }
    }
}

impl Display for TypecheckerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.kind {
            TypecheckerErrorKind::UndefinedVariable(name) => {
                write!(f, "Undefined variable: '{}'", name)
            }

            TypecheckerErrorKind::UndefinedFunction(name) => {
                write!(f, "Undefined function: '{}'", name)
            }

            TypecheckerErrorKind::UnableToResolveType(name) => {
                write!(f, "Unable to resolve type: '{}'", name)
            }

            TypecheckerErrorKind::MismatchedType { expected, received } => {
                write!(f, "Expected type '{}', but got '{}'", expected, received)
            }

            TypecheckerErrorKind::ExpectedReturn => {
                write!(f, "Expected `return`")
            }
        }
    }
}
