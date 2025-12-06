use petal_core::string_intern::StringReference;

use crate::expression::ExpressionNodeKind;

/// A string literal.
#[derive(Debug, PartialEq, Clone)]
pub struct StringLiteral {
    /// The value of the string.
    pub value: StringReference,
}

impl StringLiteral {
    /// Instantiates a new [StringLiteral].
    pub fn new(value: StringReference) -> Self {
        StringLiteral { value }
    }
}

impl Into<ExpressionNodeKind> for StringLiteral {
    fn into(self) -> ExpressionNodeKind {
        ExpressionNodeKind::StringLiteral(self)
    }
}
