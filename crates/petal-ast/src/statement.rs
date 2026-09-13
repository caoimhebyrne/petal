use petal_span::Span;

use crate::Expression;

/// A statement.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Statement {
    Return(ReturnStatement),
}

impl Statement {
    /// Return the [`Span`] that this statement covers.
    pub fn span(&self) -> Span {
        match self {
            Statement::Return(return_statement) => return_statement.span,
        }
    }
}

/// Returns a value from a block.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReturnStatement {
    /// The value being returned.
    pub value: Option<Expression>,

    /// The [`Span`] within the source code that this [`ReturnStatement`] was at.
    pub span: Span,
}

/// A block of [`Statement`]s.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Block {
    /// The [`Statement`]s in this block.
    pub statements: Vec<Statement>,

    /// The [`Span`] that the block covers.
    pub span: Span,
}
