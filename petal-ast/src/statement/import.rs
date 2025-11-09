use petal_core::string_intern::StringReference;

use crate::statement::StatementKind;

#[derive(Debug, Clone, PartialEq)]
pub struct ImportStatement {
    /// The name of the module being imported.
    pub module_name: StringReference,
}

impl ImportStatement {
    /// Creates a new [ImportStatement].
    pub fn new(module_name: StringReference) -> Self {
        ImportStatement { module_name }
    }
}

/// Allows `.into()` to be called on a [ImportStatement] to turn it into a [StatementKind].
impl From<ImportStatement> for StatementKind {
    fn from(value: ImportStatement) -> Self {
        StatementKind::ImportStatement(value)
    }
}
