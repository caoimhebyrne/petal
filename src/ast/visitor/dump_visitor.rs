use crate::{
    ast::{statement::Statement, visitor::ASTVisitor},
    core::error::Result,
};

/// An [ASTVisitor] which prints information about the visited nodes to standard output.
pub struct DumpASTVisitor {}

impl DumpASTVisitor {
    /// Creates a new [DumpASTVisitor] instance.
    pub fn new() -> Self {
        DumpASTVisitor {}
    }
}

impl ASTVisitor for DumpASTVisitor {
    fn visit(&self, statement: &mut Statement) -> Result<()> {
        println!("{:#?}", statement);

        Ok(())
    }
}
