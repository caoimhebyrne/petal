use crate::{
    ast::statement::{
        function_declaration::FunctionDeclaration, r#return::ReturnStatement, variable_declaration::VariableDeclaration,
    },
    core::source_span::SourceSpan,
};

pub mod function_declaration;
pub mod r#return;
pub mod variable_declaration;

/// A statement can be seen as an action that does not return a value.
#[derive(Debug, Clone, PartialEq)]
pub struct Statement {
    /// The kind of statement that this is.
    pub kind: StatementKind,

    /// The span within the source code that this statement was defined at.
    pub span: SourceSpan,
}

impl Statement {
    /// Creates a new statement with the provided kind of type [K] and [SourceSpan].
    pub fn new<K: Into<StatementKind>>(kind: K, span: SourceSpan) -> Self {
        Statement {
            kind: kind.into(),
            span,
        }
    }
}

/// The different kinds of statements that exist.
#[derive(Debug, Clone, PartialEq)]
pub enum StatementKind {
    /// A variable declaration statement, e.g: `let <identifier> = <expression>;`
    VariableDeclaration(VariableDeclaration),

    /// A function declaration statement, e.g: `func <name>() { <body> }`
    FunctionDeclaration(FunctionDeclaration),

    /// A return statement, e.g. `return <value>;`
    ReturnStatement(ReturnStatement),
}
