use crate::{
    ast::expression::{
        binary_operation::BinaryOperation,
        function_call::FunctionCall,
        structure_initialization::StructureInitialization,
    },
    core::span::Span,
};

pub mod binary_operation;
pub mod function_call;
pub mod structure_initialization;

/// An expression node within the abstract syntax tree.
/// This node kind should always emit a value once evaluated.
#[derive(Debug, Clone, PartialEq)]
pub struct Expression {
    /// The kind of expression that this is.
    pub kind: ExpressionKind,

    /// The [Span] that this expression occured at within the source file.
    pub span: Span,
}

impl Expression {
    /// Creates a new [Expression].
    pub fn new(kind: ExpressionKind, span: Span) -> Self {
        Self { kind, span }
    }
}

/// The different kinds of [Expression]s that are available in an abstract syntax tree.
#[derive(Debug, Clone, PartialEq)]
pub enum ExpressionKind {
    /// A number literal.
    NumberLiteral(f64),

    /// A boolean literal.
    BooleanLiteral(bool),

    /// An identifier reference.
    IdentifierReference(String),

    /// A reference expression.
    Reference(Box<Expression>),

    /// A dereference expression.
    Dereference(Box<Expression>),

    /// A function call.
    FunctionCall(FunctionCall),

    /// A binary operation.
    BinaryOperation(BinaryOperation),

    /// A structure initialization.
    StructureInitialization(StructureInitialization),
}
