use super::Node;
use crate::typechecker::r#type::Type;

#[derive(Debug, Clone)]
pub enum NodeKind {
    IntegerLiteral(IntegerLiteralNode),
    IdentifierReference(IdentifierReferenceNode),
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
