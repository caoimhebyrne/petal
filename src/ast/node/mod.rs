use crate::core::location::Location;
use kind::NodeKind;

pub mod kind;

#[derive(Debug, Clone)]
pub struct Node {
    pub kind: NodeKind,
    pub location: Location,
}

impl Node {
    pub fn new(kind: NodeKind, location: Location) -> Node {
        Node { kind, location }
    }
}
