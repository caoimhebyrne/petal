use crate::core::location::Location;

pub mod expression;
pub mod operator;
pub mod statement;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Node {
    pub location: Location,
}

impl Node {
    pub fn new(location: Location) -> Self {
        Self { location }
    }
}
