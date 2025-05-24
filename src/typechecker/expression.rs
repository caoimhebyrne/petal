use crate::{
    ast::node::kind::{BinaryOperationNode, FunctionCallNode, IdentifierReferenceNode, IntegerLiteralNode},
    core::location::Location,
};

use super::{
    Typechecker,
    context::TypecheckerContext,
    error::TypecheckerError,
    r#type::{Type, kind::TypeKind},
};

pub trait ExpressionTypecheck {
    fn resolve(
        &mut self,
        context: &mut TypecheckerContext,
        expected_type: Option<&Type>,
        location: Location,
    ) -> Result<Type, TypecheckerError>;
}

impl ExpressionTypecheck for IntegerLiteralNode {
    fn resolve(
        &mut self,
        _context: &mut TypecheckerContext,
        expected_type: Option<&Type>,
        location: Location,
    ) -> Result<Type, TypecheckerError> {
        let integer_type = match expected_type {
            Some(r#type) if matches!(r#type.kind, TypeKind::Integer(_)) => r#type.clone(),
            _ => Type::new(TypeKind::Integer(32), Some(location)),
        };

        self.r#type = Some(integer_type.clone());

        Ok(integer_type)
    }
}

impl ExpressionTypecheck for IdentifierReferenceNode {
    fn resolve(
        &mut self,
        context: &mut TypecheckerContext,
        _expected_type: Option<&Type>,
        location: Location,
    ) -> Result<Type, TypecheckerError> {
        let function_scope = match &context.function_scope {
            Some(value) => value,
            None => panic!("Identifier reference outside of function scope?"),
        };

        let variable_type = function_scope
            .variables
            .get(&self.name)
            .ok_or(TypecheckerError::undefined_variable(self.name.clone(), Some(location)))
            .cloned()?;

        self.r#type = Some(variable_type.clone());
        Ok(variable_type)
    }
}

impl ExpressionTypecheck for BinaryOperationNode {
    fn resolve(
        &mut self,
        context: &mut TypecheckerContext,
        expected_type: Option<&Type>,
        _location: Location,
    ) -> Result<Type, TypecheckerError> {
        // The types of both the left and right-hand sides of the expressions must be resolvable.
        let left_type = Typechecker::check_expression(&mut self.left, context, expected_type)?;
        let right_type = Typechecker::check_expression(&mut self.right, context, expected_type)?;

        // The left and right sides of the expression must be of the same type.
        // TODO: Add the ability to declare types as compatible with each-other for operations.
        if left_type.kind != right_type.kind {
            return Err(TypecheckerError::mismatched_type(
                left_type.kind,
                right_type.kind,
                Some(self.right.location),
            ));
        }

        self.value_type = Some(left_type.clone());
        Ok(left_type)
    }
}

impl ExpressionTypecheck for FunctionCallNode {
    fn resolve(
        &mut self,
        context: &mut TypecheckerContext,
        _expected_type: Option<&Type>,
        location: Location,
    ) -> Result<Type, TypecheckerError> {
        // To call a function, we need to know its return type.
        let return_type = context
            .functions
            .get(&self.name)
            .ok_or(TypecheckerError::undefined_function(self.name.clone(), Some(location)))?;

        self.return_type = Some(return_type.clone());
        Ok(return_type.clone())
    }
}
