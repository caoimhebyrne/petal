use crate::core::location::Location;
use kind::TypeKind;
use std::fmt::Display;

pub mod kind;

#[derive(Debug, Clone, PartialEq)]
pub struct Type {
    pub kind: TypeKind,
    pub location: Location,
}

impl Type {
    pub fn new(kind: TypeKind, location: Location) -> Self {
        Self { kind, location }
    }
}

impl Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.kind.fmt(f)
    }
}
