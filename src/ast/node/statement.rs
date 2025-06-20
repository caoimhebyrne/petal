use super::{
    Node,
    expression::{Expression, FunctionCall},
    extra::FunctionParameter,
};
use crate::typechecker::r#type::Type;

#[derive(Debug, Clone)]
pub enum Statement {
    FunctionDefinition(FunctionDefinition),
    VariableDeclaration(VariableDeclaration),
    Return(Return),
    VariableReassignment(VariableReassignment),
    FunctionCall(FunctionCall),
    If(If),
}

// A function definition node.
#[derive(Debug, Clone)]
pub struct FunctionDefinition {
    // The node associated with this statement.
    pub node: Node,

    // The name of the function.
    pub name: String,

    // Whether the function is an external function or not.
    pub is_extern: bool,

    // The parameters of the function.
    pub parameters: Vec<FunctionParameter>,

    // The return type of the function.
    pub return_type: Option<Type>,

    // The body of the function.
    pub body: Vec<Statement>,
}

// A variable declaration node.
#[derive(Debug, Clone)]
pub struct VariableDeclaration {
    // The node associated with this statement.
    pub node: Node,

    // The name of the variable.
    pub name: String,

    // The variable's declared type.
    pub declared_type: Type,

    // The value assigned to the variable at declaration.
    pub value: Expression,
}

// A return node.
#[derive(Debug, Clone)]
pub struct Return {
    // The node associated with this statement.
    pub node: Node,

    // The (optional) value being returned.
    pub value: Option<Expression>,
}

// A variable re-assignment statement.
#[derive(Debug, Clone)]
pub struct VariableReassignment {
    // The node associated with this statement.
    pub node: Node,

    // The name of the variable being assigned to.
    pub name: String,

    // The value being assigned to the variable.
    pub value: Expression,
}

#[derive(Debug, Clone)]
pub struct If {
    // The node associated with this statement.
    pub node: Node,

    // The condition that must be true for the code within the block to be executed.
    pub condition: Expression,

    // The block to run if the condition is true.
    pub block: Vec<Statement>,
}

impl Statement {
    pub fn node(&self) -> Node {
        match self {
            Statement::FunctionCall(function_call) => function_call.node,
            Statement::FunctionDefinition(function_definition) => function_definition.node,
            Statement::If(r#if) => r#if.node,
            Statement::Return(r#return) => r#return.node,
            Statement::VariableDeclaration(variable_declaration) => variable_declaration.node,
            Statement::VariableReassignment(variable_reassignment) => variable_reassignment.node,
        }
    }
}
