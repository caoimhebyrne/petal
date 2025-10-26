use crate::{ast::statement::Statement, core::error::Result};

pub mod dump_visitor;

/// A trait for all types that are interested in consuming AST nodes from a [StatementStream] to implement.
pub trait ASTVisitor {
    fn visit(&self, statement: &mut Statement) -> Result<()>;
}
