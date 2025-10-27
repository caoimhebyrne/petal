use petal_core::error::Result;

use crate::{statement::Statement, visitor::ASTVisitor};

/// Wraps a [Vec] of AST [Statement]s, allowing the caller to pass a ASTVisitor to consume the [Vec] of [Statement]s.
pub struct StatementStream {
    /// The [Vec] containing the [Statement]s that this stream wraps.
    statements: Vec<Statement>,
}

impl StatementStream {
    /// Returns a new [StatementStream] instance from a [Vec] of [Statement]s.
    pub fn new(statements: Vec<Statement>) -> Self {
        StatementStream { statements }
    }

    /// Calls [ASTVisitor.visitStatement] on the provided [visitor] for all statements held within this stream.
    pub fn visit(&mut self, visitor: &mut dyn ASTVisitor) -> Result<()> {
        for statement in &mut self.statements {
            visitor.visit(statement)?;
        }

        Ok(())
    }
}
