use crate::ast::node::{expression::Expression, statement::Statement};
use context::TypecheckerContext;
use error::TypecheckerError;
use expression::ExpressionTypecheck;
use statement::StatementTypecheck;
use r#type::{Type, kind::TypeKind};

pub mod context;
pub mod error;
pub mod expression;
pub mod statement;
pub mod r#type;

pub struct Typechecker<'a> {
    context: TypecheckerContext,
    nodes: &'a mut Vec<Statement>,
}

impl<'a> Typechecker<'a> {
    pub fn new(nodes: &'a mut Vec<Statement>) -> Self {
        Self {
            context: TypecheckerContext::new(),
            nodes,
        }
    }

    pub fn check(&mut self) -> Result<(), TypecheckerError> {
        Typechecker::check_block(self.nodes, &mut self.context)
    }

    pub fn check_block(block: &mut Vec<Statement>, context: &mut TypecheckerContext) -> Result<(), TypecheckerError> {
        for node in block {
            Typechecker::check_statement(node, context)?;
        }

        Ok(())
    }

    pub fn check_statement(
        statement: &mut Statement,
        context: &mut TypecheckerContext,
    ) -> Result<(), TypecheckerError> {
        match statement {
            Statement::VariableDeclaration(variable_declaration) => variable_declaration.resolve(context),
            Statement::FunctionDefinition(function_definition) => function_definition.resolve(context),
            Statement::Return(r#return) => r#return.resolve(context),
            Statement::VariableReassignment(variable_reassignment) => variable_reassignment.resolve(context),
            Statement::FunctionCall(function_call) => function_call.resolve(context, None).map(|_| {}),
            Statement::If(r#if) => r#if.resolve(context),
        }
    }

    pub fn check_expression(
        expression: &mut Expression,
        context: &mut TypecheckerContext,
        expected_type: Option<&Type>,
    ) -> Result<Type, TypecheckerError> {
        match expression {
            Expression::IntegerLiteral(integer_literal) => integer_literal.resolve(context, expected_type),
            Expression::StringLiteral(string_literal) => string_literal.resolve(context, expected_type),
            Expression::BinaryOperation(binary_operation) => binary_operation.resolve(context, expected_type),
            Expression::FunctionCall(function_call) => function_call.resolve(context, expected_type),
            Expression::BinaryComparison(binary_comparison) => binary_comparison.resolve(context, expected_type),
            Expression::IdentifierReference(identifier_reference) => {
                identifier_reference.resolve(context, expected_type)
            }
            Expression::BooleanLiteral(boolean_literal) => boolean_literal.resolve(context, expected_type),
        }
    }

    pub fn resolve_type(r#type: Type) -> Result<Type, TypecheckerError> {
        if let TypeKind::Reference(referenced) = r#type.kind {
            let resolved = Typechecker::resolve_type(Type::new(*referenced, r#type.location))?;
            return Ok(Type::new(TypeKind::Reference(Box::new(resolved.kind)), r#type.location));
        }

        let name = match r#type.kind {
            TypeKind::Unresolved(name) => name,
            _ => return Ok(r#type),
        };

        let resolved_kind = match name.as_str() {
            "i8" => TypeKind::Integer(8),
            "i32" => TypeKind::Integer(32),
            "bool" => TypeKind::Boolean,

            _ => {
                return Err(TypecheckerError::unable_to_resolve_type(name, r#type.location));
            }
        };

        Ok(Type::new(resolved_kind, r#type.location))
    }
}
