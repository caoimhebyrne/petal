use error::TypecheckerError;
use expression::ExpressionTypecheck;
use statement::StatmentTypecheck;
use r#type::{Type, kind::TypeKind};

use crate::ast::node::{Node, kind::NodeKind};

pub mod error;
pub mod expression;
pub mod statement;
pub mod r#type;

pub struct Typechecker<'a> {
    nodes: &'a mut Vec<Node>,
}

impl<'a> Typechecker<'a> {
    pub fn new(nodes: &'a mut Vec<Node>) -> Typechecker<'a> {
        return Typechecker { nodes };
    }

    pub fn check(&mut self) -> Result<(), TypecheckerError> {
        Typechecker::check_block(&mut self.nodes)
    }

    pub fn check_block(block: &mut Vec<Node>) -> Result<(), TypecheckerError> {
        for node in block {
            Typechecker::check_statement(node)?;
        }

        Ok(())
    }

    pub fn check_statement(statement: &mut Node) -> Result<(), TypecheckerError> {
        match &mut statement.kind {
            NodeKind::VariableDeclaration(variable_declaration) => variable_declaration.resolve(),
            NodeKind::FunctionDefinition(function_definition) => function_definition.resolve(),
            NodeKind::Return(r#return) => r#return.resolve(),

            _ => todo!(),
        }
    }

    pub fn check_expression(
        expression: &mut Node,
        expected_type: Option<&Type>,
    ) -> Result<Type, TypecheckerError> {
        match &mut expression.kind {
            NodeKind::IntegerLiteral(integer_literal) => integer_literal.resolve(expected_type),
            _ => todo!(),
        }
    }

    pub fn resolve_type(r#type: Type) -> Result<Type, TypecheckerError> {
        let name = match r#type.kind {
            TypeKind::Unresolved(name) => name,
            _ => return Ok(r#type),
        };

        let resolved_kind = match name.as_str() {
            "i32" => TypeKind::Integer(32),

            _ => {
                return Err(TypecheckerError::unable_to_resolve_type(
                    name,
                    r#type.location,
                ));
            }
        };

        Ok(Type::new(resolved_kind, r#type.location))
    }
}
