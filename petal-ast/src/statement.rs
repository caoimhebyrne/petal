///! There are two kinds of statement nodes: regular statement nodes and top level statement nodes.
///! A statement node does not produce a value, unlike an expression node which does produce a value.
use crate::{
    node::FunctionCall,
    statement::{
        function_declaration::FunctionDeclaration, import::Import, r#return::Return,
        variable_assignment::VariableAssignment, variable_declaration::VariableDeclaration,
    },
};
use petal_core::source_span::SourceSpan;

pub mod function_declaration;
pub mod import;
pub mod r#return;
pub mod variable_assignment;
pub mod variable_declaration;

/// A top-level statement node.
#[derive(Debug, PartialEq, Clone)]
pub struct TopLevelStatementNode {
    /// The kind of [TopLevelStatementNode] that this is.
    pub kind: TopLevelStatementNodeKind,

    /// The position that this node occurred at within the source file.
    pub span: SourceSpan,
}

impl TopLevelStatementNode {
    /// Instantiates a new [TopLevelStatementNode].
    pub fn new(kind: impl Into<TopLevelStatementNodeKind>, span: SourceSpan) -> Self {
        TopLevelStatementNode {
            kind: kind.into(),
            span,
        }
    }

    /// Instantiates a new [TopLevelStatementNode].
    pub fn from_pair(pair: (impl Into<TopLevelStatementNodeKind>, SourceSpan)) -> Self {
        TopLevelStatementNode::new(pair.0, pair.1)
    }
}

/// The different kinds of [TopLevelStatementNode]s that are recognized.
#[derive(Debug, PartialEq, Clone)]
pub enum TopLevelStatementNodeKind {
    /// A function declaration statement.
    FunctionDeclaration(FunctionDeclaration),

    /// An import statement.
    Import(Import),
}

/// A statement node.
/// This statement node can be at any point within the AST, outside of the top level.
#[derive(Debug, PartialEq, Clone)]
pub struct StatementNode {
    /// The kind of [StatementNode] that this is.
    pub kind: StatementNodeKind,

    /// The position that this node occurred at within the source file.
    pub span: SourceSpan,
}

impl StatementNode {
    /// Instantiates a new [StatementNode].
    pub fn new(kind: impl Into<StatementNodeKind>, span: SourceSpan) -> Self {
        StatementNode {
            kind: kind.into(),
            span,
        }
    }

    // Instantiates a new [StatementNode].
    pub fn from_pair(pair: (impl Into<StatementNodeKind>, SourceSpan)) -> Self {
        StatementNode::new(pair.0, pair.1)
    }
}

/// The different kinds of [StatementNode]s that are recognized.
#[derive(Debug, PartialEq, Clone)]
pub enum StatementNodeKind {
    /// A return statement.
    Return(Return),

    /// A variable declaration statement.
    VariableDeclaration(VariableDeclaration),

    /// A variable assignment.
    VariableAssignment(VariableAssignment),

    /// A function call.
    FunctionCall(FunctionCall),
}
