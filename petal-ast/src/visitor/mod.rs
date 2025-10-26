use petal_core::error::Result;

use crate::statement::Statement;

pub mod dump_visitor;

/// A trait for all types that are interested in consuming AST nodes from a [StatementStream] to implement.
pub trait ASTVisitor {
    fn visit(&self, statement: &mut Statement) -> Result<()>;
}
