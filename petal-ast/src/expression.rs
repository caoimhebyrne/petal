use std::collections::HashMap;

use petal_core::{source_span::SourceSpan, string_intern::StringReference, r#type::TypeReference};

use crate::statement::function_call::FunctionCall;

/// An expression can be seen as an action that can return a value.
#[derive(Debug, Clone, PartialEq)]
pub struct Expression {
    /// The kind of expression that this is.
    pub kind: ExpressionKind,

    /// The type of the value that this expression produces.
    pub r#type: Option<TypeReference>,

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

    /// A string literal.
    StringLiteral(StringReference),

    /// A reference to an identifier.
    IdentifierReference(IdentifierReference),

    /// A binary operation between two other expressions.
    BinaryOperation(BinaryOperation),

    /// A function call, e.g. `<name>()`.
    FunctionCall(FunctionCall),

    /// A structure initialization. e.g. `{ field = value }`
    StructureInitialization(StructureInitialization),
}

/// A reference to an identifier.
#[derive(Debug, Clone, PartialEq)]
pub struct IdentifierReference {
    /// The identifier.
    pub name: StringReference,

    /// Whether it is being caputured as a reference or not.
    pub is_reference: bool,
}

impl IdentifierReference {
    /// Creates a new [IdentifierReference].
    pub fn new(name: StringReference, is_reference: bool) -> Self {
        IdentifierReference { name, is_reference }
    }
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

/// A structure initialization.
#[derive(Debug, Clone, PartialEq)]
pub struct StructureInitialization {
    /// The names of the structure fields mapped to their values.
    pub fields: HashMap<StringReference, Expression>,
}

impl StructureInitialization {
    /// Creates a new [StructureInitialization].
    pub fn new(fields: HashMap<StringReference, Expression>) -> Self {
        StructureInitialization { fields }
    }
}
