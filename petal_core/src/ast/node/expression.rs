use super::{
    Node,
    operator::{Comparison, Operation},
};
use crate::typechecker::r#type::Type;

#[derive(Debug, Clone)]
pub enum Expression {
    IntegerLiteral(IntegerLiteral),
    StringLiteral(StringLiteral),
    BooleanLiteral(BooleanLiteral),
    IdentifierReference(IdentifierReference),
    FunctionCall(FunctionCall),
    BinaryOperation(BinaryOperation),
    BinaryComparison(BinaryComparison),
}

// A literal integer in the source.
#[derive(Debug, Clone)]
pub struct IntegerLiteral {
    // The node associated with this expression.
    pub node: Node,

    // The literal value of the integer.
    pub value: u64,

    // The expected type of the integer.
    pub expected_type: Option<Type>,
}

// A literal string in the source.
#[derive(Debug, Clone)]
pub struct StringLiteral {
    // The node associated with this expression.
    pub node: Node,

    // The literal value of the string.
    pub value: String,
}

// A boolean literal in the source.
#[derive(Debug, Clone)]
pub struct BooleanLiteral {
    // The node associated with this expression.
    pub node: Node,

    // The literal value of the string.
    pub value: bool,
}

// An identifier reference.
#[derive(Debug, Clone)]
pub struct IdentifierReference {
    // The node associated with this expression.
    pub node: Node,

    // The name of the variable being referenced.
    pub name: String,

    // The expected type of the variable.
    pub expected_type: Option<Type>,

    // Whether the identifier is being passed by reference.
    pub is_reference: bool,
}

// A function call.
#[derive(Debug, Clone)]
pub struct FunctionCall {
    // The node associated with this expression.
    pub node: Node,

    // The name of the function being called.
    pub name: String,

    // The arguments being passed to the function.
    pub arguments: Vec<Expression>,

    // The expected return type of the function.
    pub expected_type: Option<Type>,
}

// A binary operation between two nodes.
#[derive(Debug, Clone)]
pub struct BinaryOperation {
    // The node associated with this expression.
    pub node: Node,

    // The operation to perform between the two values.
    pub operation: Operation,

    // The left-hand side of the expression.
    pub left: Box<Expression>,

    // The right-hand side of the expression.
    pub right: Box<Expression>,

    // The expected type to be produced by this binary operation.
    pub expected_type: Option<Type>,
}

// A binary comparison between two nodes.
#[derive(Debug, Clone)]
pub struct BinaryComparison {
    // The node associated with this expression.
    pub node: Node,

    // The comparison to perform between the two values.
    pub comparison: Comparison,

    // The left-hand side of the expression.
    pub left: Box<Expression>,

    // The right-hand side of the expression.
    pub right: Box<Expression>,
}

impl Expression {
    pub fn node(&self) -> Node {
        match self {
            Expression::BinaryComparison(binary_comparison) => binary_comparison.node,
            Expression::BinaryOperation(binary_operation) => binary_operation.node,
            Expression::BooleanLiteral(boolean_literal) => boolean_literal.node,
            Expression::FunctionCall(function_call) => function_call.node,
            Expression::IdentifierReference(identifier_reference) => identifier_reference.node,
            Expression::IntegerLiteral(integer_literal) => integer_literal.node,
            Expression::StringLiteral(string_literal) => string_literal.node,
        }
    }
}
