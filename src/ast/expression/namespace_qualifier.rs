use crate::ast::expression::ExpressionKind;

#[derive(Debug, Clone, PartialEq)]
pub struct NamespaceQualifier {
    /// The identifier of the namespace.
    pub namespace: String,

    /// The identifier being qualified.
    pub identifier: String,
}

impl NamespaceQualifier {
    /// Creates a new [`NamespaceQualifier`].
    pub fn new(namespace: String, identifier: String) -> Self {
        Self { namespace, identifier }
    }
}

impl From<NamespaceQualifier> for ExpressionKind {
    fn from(value: NamespaceQualifier) -> Self {
        Self::NamespaceQualifier(value)
    }
}
