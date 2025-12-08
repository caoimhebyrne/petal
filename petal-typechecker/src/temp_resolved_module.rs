use std::{
    path::PathBuf,
    sync::atomic::{AtomicUsize, Ordering},
};

use petal_ast::statement::TopLevelStatementNode;

// FIXME: This shouldn't be here, but it won't work in core because AST depends on core.

static GLOBAL_MODULE_ID: AtomicUsize = AtomicUsize::new(0);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ModuleId(usize);

impl ModuleId {
    pub fn next() -> Self {
        ModuleId(GLOBAL_MODULE_ID.fetch_add(1, Ordering::Relaxed))
    }
}

/// A module which has been resolved by the compiler.
#[derive(Debug, Clone)]
pub struct ResolvedModule {
    /// The unique identifier for this module.
    pub id: ModuleId,

    /// The path that the module was defined at.
    pub source_path: PathBuf,

    /// The contents of the source.
    pub source_contents: String,

    /// The AST nodes involved in this module.
    pub statements: Vec<TopLevelStatementNode>,
}

impl ResolvedModule {
    /// Creates a new [ResolvedModule].
    pub fn new(
        module_id: ModuleId,
        source_path: PathBuf,
        source_contents: String,
        statements: Vec<TopLevelStatementNode>,
    ) -> Self {
        ResolvedModule {
            id: module_id,
            source_path,
            source_contents,
            statements,
        }
    }
}
