use crate::{
    ast::statement::{
        function_declaration::FunctionDeclaration,
        r#return::Return,
    },
    core::span::Span,
};

pub mod function_declaration;
pub mod r#return;

/// A statement node within the abstract syntax tree.
/// This node kind typically does not yield a value once evaluated.
#[derive(Debug, Clone, PartialEq)]
pub struct Statement {
    /// The kind of statement that this is.
    pub kind: StatementKind,

    /// The [Span] that this statement occured at within the source file.
    pub span: Span,
}

impl Statement {
    /// Creates a new [Statement].
    pub fn new(kind: StatementKind, span: Span) -> Self {
        Statement { kind, span }
    }

    /// Creates a new [Statement] from the provided [Into<StatementKind>] kind.
    pub fn from(kind: impl Into<StatementKind>, span: Span) -> Self {
        Self::new(kind.into(), span)
    }
}

/// The different kinds of [Statement]s in the abstract syntax tree.
#[derive(Debug, Clone, PartialEq)]
pub enum StatementKind {
    /// A function declaration statement.
    FunctionDeclaration(FunctionDeclaration),

    /// A return statement.
    Return(Return),
}
