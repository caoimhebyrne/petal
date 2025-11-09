use std::path::PathBuf;

use petal_ast::statement::Statement;

// FIXME: This shouldn't be here, but it won't work in core because AST depends on core.

/// A module which has been resolved by the compiler.
#[derive(Debug, Clone)]
pub struct ResolvedModule {
    /// The path that the module was defined at.
    pub source_path: PathBuf,

    /// The contents of the source.
    pub source_contents: String,

    /// The AST nodes involved in this module.
    pub statements: Vec<Statement>,
}

impl ResolvedModule {
    /// Creates a new [ResolvedModule].
    pub fn new(source_path: PathBuf, source_contents: String, statements: Vec<Statement>) -> Self {
        ResolvedModule {
            source_path,
            source_contents,
            statements,
        }
    }
}
