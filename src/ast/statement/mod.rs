use crate::{
    ast::{
        expression::function_call::FunctionCall,
        statement::{
            function_declaration::FunctionDeclaration,
            r#if::If,
            import::Import,
            r#return::Return,
            variable_assignment::VariableAssignment,
            variable_declaration::VariableDeclaration,
        },
    },
    core::span::Span,
};

pub mod function_declaration;
pub mod r#if;
pub mod import;
pub mod r#return;
pub mod variable_assignment;
pub mod variable_declaration;

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

    /// A function call statement.
    FunctionCall(FunctionCall),

    /// An if statement.
    If(If),

    /// A return statement.
    Return(Return),

    /// A variable declaration statement.
    VariableDeclaration(VariableDeclaration),

    /// A variable assignment statement.
    VariableAssignment(VariableAssignment),

    /// An import statement.
    Import(Import),
}
