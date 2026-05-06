use crate::ast::statement::{
    Statement,
    StatementKind,
};

#[derive(Debug, Clone, PartialEq)]
pub struct NamespaceDeclaration {
    /// The name of the namespace being declared.
    pub name: String,

    /// The declarations within this namespace.
    pub body: Vec<Statement>,
}

impl NamespaceDeclaration {
    /// Creates a new [`NamespaceDeclaration`].
    pub fn new(name: String, body: Vec<Statement>) -> NamespaceDeclaration {
        NamespaceDeclaration { name, body }
    }
}

impl From<NamespaceDeclaration> for StatementKind {
    fn from(value: NamespaceDeclaration) -> Self {
        Self::NamespaceDeclaration(value)
    }
}
