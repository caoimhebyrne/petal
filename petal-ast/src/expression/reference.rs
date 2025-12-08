use crate::expression::{ExpressionNode, ExpressionNodeKind};

/// An expression which indicates that a reference should be taken of another expression.
#[derive(Debug, PartialEq, Clone)]
pub struct Reference {
    /// The value that a reference should be taken to.
    pub value: Box<ExpressionNode>,
}

impl Reference {
    /// Instantiates a new [Reference].
    pub fn new(value: ExpressionNode) -> Self {
        Reference { value: Box::new(value) }
    }
}

impl From<Reference> for ExpressionNodeKind {
    fn from(val: Reference) -> Self {
        ExpressionNodeKind::Reference(val)
    }
}
