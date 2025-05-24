use super::{Node, operator::Operation};
use crate::typechecker::r#type::Type;

#[derive(Debug, Clone)]
pub enum Expression {
    IntegerLiteral(IntegerLiteral),
    IdentifierReference(IdentifierReference),
    FunctionCall(FunctionCall),
    BinaryOperation(BinaryOperation),
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

// An identifier reference.
#[derive(Debug, Clone)]
pub struct IdentifierReference {
    // The node associated with this expression.
    pub node: Node,

    // The name of the variable being referenced.
    pub name: String,

    // The expected type of the variable.
    pub expected_type: Option<Type>,
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
