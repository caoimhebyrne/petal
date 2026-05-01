use crate::ast::statement::StatementKind;

#[derive(Debug, Clone, PartialEq)]
pub struct Import {
    /// The name of the module being imported.
    pub name: String,
}

impl Import {
    /// Creates a new [`Import`] statement.
    pub fn new(name: String) -> Self {
        Self { name }
    }
}

impl From<Import> for StatementKind {
    fn from(value: Import) -> Self {
        Self::Import(value)
    }
}
