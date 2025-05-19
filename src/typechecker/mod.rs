use context::TypecheckerContext;
use error::TypecheckerError;
use expression::ExpressionTypecheck;
use statement::StatmentTypecheck;
use r#type::{Type, kind::TypeKind};

use crate::ast::node::{Node, kind::NodeKind};

pub mod context;
pub mod error;
pub mod expression;
pub mod statement;
pub mod r#type;

pub struct Typechecker<'a> {
    context: TypecheckerContext,
    nodes: &'a mut Vec<Node>,
}

impl<'a> Typechecker<'a> {
    pub fn new(nodes: &'a mut Vec<Node>) -> Typechecker<'a> {
        return Typechecker {
            context: TypecheckerContext::new(),
            nodes,
        };
    }

    pub fn check(&mut self) -> Result<(), TypecheckerError> {
        Typechecker::check_block(&mut self.nodes, &mut self.context)
    }

    pub fn check_block(
        block: &mut Vec<Node>,
        context: &mut TypecheckerContext,
    ) -> Result<(), TypecheckerError> {
        for node in block {
            Typechecker::check_statement(node, context)?;
        }

        Ok(())
    }

    pub fn check_statement(
        statement: &mut Node,
        context: &mut TypecheckerContext,
    ) -> Result<(), TypecheckerError> {
        match &mut statement.kind {
            NodeKind::VariableDeclaration(variable_declaration) => {
                variable_declaration.resolve(context)
            }

            NodeKind::FunctionDefinition(function_definition) => {
                function_definition.resolve(context)
            }

            NodeKind::Return(r#return) => r#return.resolve(context),

            _ => todo!(),
        }
    }

    pub fn check_expression(
        expression: &mut Node,
        context: &mut TypecheckerContext,
        expected_type: Option<&Type>,
    ) -> Result<Type, TypecheckerError> {
        match &mut expression.kind {
            NodeKind::IntegerLiteral(integer_literal) => {
                integer_literal.resolve(context, expected_type)
            }

            NodeKind::IdentifierReference(identifier_reference) => {
                identifier_reference.resolve(context, expected_type)
            }

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
