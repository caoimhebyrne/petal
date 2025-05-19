use crate::core::location::Location;
use std::fmt::Display;

#[derive(Debug, Clone)]
pub enum TypecheckerErrorKind {
    UnableToResolveType(String),
}

#[derive(Debug, Clone)]
pub struct TypecheckerError {
    pub kind: TypecheckerErrorKind,
    pub location: Location,
}

impl TypecheckerError {
    pub fn unable_to_resolve_type(name: String, location: Location) -> TypecheckerError {
        TypecheckerError {
            kind: TypecheckerErrorKind::UnableToResolveType(name),
            location,
        }
    }
}

impl Display for TypecheckerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.kind {
            TypecheckerErrorKind::UnableToResolveType(name) => {
                write!(f, "Unable to resolve type: '{}'", name)
            }
        }
    }
}
