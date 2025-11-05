use petal_core::{source_span::SourceSpan, string_intern::StringReference, r#type::pool::TypeId};

use crate::statement::function_call::FunctionCall;

/// An expression can be seen as an action that can return a value.
#[derive(Debug, Clone, PartialEq)]
pub struct Expression {
    /// The kind of expression that this is.
    pub kind: ExpressionKind,

    /// The type of the value that this expression produces.
    pub r#type: Option<TypeId>,

    /// The span within the source code that this expression was defined at.
    pub span: SourceSpan,
}

impl Expression {
    pub fn new(kind: ExpressionKind, span: SourceSpan) -> Self {
        Expression {
            kind,
            r#type: None,
            span,
        }
    }
}

/// The different kinds of expressions that exist.
#[derive(Debug, Clone, PartialEq)]
pub enum ExpressionKind {
    // An integer literal, e.g: `12345`.
    IntegerLiteral(u64),

    /// A reference to an identifier.
    IdentifierReference(StringReference),

    /// A binary operation between two other expressions.
    BinaryOperation(BinaryOperation),

    /// A function call, e.g. `<name>()`.
    FunctionCall(FunctionCall),
}

/// A binary operation between two [Expression]s.
///
/// The value type of a [BinaryOperation] should always be the [Expression::type] of the [BinaryOperation::left]
#[derive(Debug, Clone, PartialEq)]
pub struct BinaryOperation {
    /// The expression on the left-hand side.
    pub left: Box<Expression>,

    /// The expression on the right-hand side.
    pub right: Box<Expression>,

    /// The operation to perform on the two expressions.
    pub operation: Operation,
}

impl BinaryOperation {
    pub fn new(left: Expression, right: Expression, operation: Operation) -> Self {
        BinaryOperation {
            left: Box::new(left),
            right: Box::new(right),
            operation,
        }
    }
}

/// An operation to perform on two values within a [BinaryOperation].
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Operation {
    Add,
    Subtract,
    Multiply,
    Divide,
}
