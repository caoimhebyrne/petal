use super::{Node, operator::BinaryOperation};
use crate::typechecker::r#type::Type;

#[derive(Debug, Clone)]
pub enum NodeKind {
    IntegerLiteral(IntegerLiteralNode),
    IdentifierReference(IdentifierReferenceNode),
    FunctionCall(FunctionCallNode),
    BinaryOperation(BinaryOperationNode),
    FunctionDefinition(FunctionDefinitionNode),
    VariableDeclaration(VariableDeclarationNode),
    Return(ReturnNode),
}

// An integer literal node.
#[derive(Debug, Clone)]
pub struct IntegerLiteralNode {
    pub value: u64,
    pub r#type: Option<Type>,
}

// An identifier reference node.
#[derive(Debug, Clone)]
pub struct IdentifierReferenceNode {
    pub name: String,
    pub r#type: Option<Type>,
}

// A function call node.
#[derive(Debug, Clone)]
pub struct FunctionCallNode {
    // The name of the function being called.
    pub name: String,

    // The expected return type of the function.
    pub return_type: Option<Type>,
}

// A binary operation between two nodes.
#[derive(Debug, Clone)]
pub struct BinaryOperationNode {
    // The operation to perform between the two values.
    pub operation: BinaryOperation,

    // The left-hand side of the expression.
    pub left: Box<Node>,

    // The right-hand side of the expression.
    pub right: Box<Node>,

    // The expected type to be produced by this binary operation.
    pub value_type: Option<Type>,
}

// A function definition node.
#[derive(Debug, Clone)]
pub struct FunctionDefinitionNode {
    // The name of the function.
    pub name: String,

    // The return type of the function.
    pub return_type: Option<Type>,

    // The body of the function.
    pub body: Vec<Node>,
}

// A variable declaration node.
#[derive(Debug, Clone)]
pub struct VariableDeclarationNode {
    // The name of the variable.
    pub name: String,

    // The variable's declared type.
    pub declared_type: Type,

    // The value assigned to the variable at declaration.
    pub value: Box<Node>,
}

// A return node.
#[derive(Debug, Clone)]
pub struct ReturnNode {
    // The (optional) value being returned.
    pub value: Option<Box<Node>>,
}
