use petal_core::string_intern::StringReference;

use crate::statement::TopLevelStatementNodeKind;

/// A top-level statement that indicates that another module should be imported.
#[derive(Debug, PartialEq, Clone)]
pub struct Import {
    /// The name of the module to import.
    pub name: StringReference,
}

impl Import {
    /// Instantiates a new [Import].
    pub fn new(name: StringReference) -> Self {
        Import { name }
    }
}

impl From<Import> for TopLevelStatementNodeKind {
    fn from(val: Import) -> Self {
        TopLevelStatementNodeKind::Import(val)
    }
}
