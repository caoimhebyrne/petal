use crate::expression::ExpressionNodeKind;
use petal_core::string_intern::StringReference;

/// A reference to a variable by its name (identifier).
#[derive(Debug, PartialEq, Clone)]
pub struct IdentifierReference {
    /// The name of the variable.
    pub identifier: StringReference,
}

impl IdentifierReference {
    /// Instantiates a new [IdentifierReference].
    pub fn new(identifier: StringReference) -> Self {
        IdentifierReference { identifier }
    }
}

impl From<IdentifierReference> for ExpressionNodeKind {
    fn from(val: IdentifierReference) -> Self {
        ExpressionNodeKind::IdentifierReference(val)
    }
}
