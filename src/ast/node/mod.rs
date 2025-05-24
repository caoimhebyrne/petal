use crate::core::location::Location;
use kind::NodeKind;

pub mod kind;
pub mod operator;

#[derive(Debug, Clone)]
pub struct Node {
    pub kind: NodeKind,
    pub location: Location,
}

impl Node {
    pub fn new(kind: NodeKind, location: Location) -> Self {
        Self { kind, location }
    }
}
