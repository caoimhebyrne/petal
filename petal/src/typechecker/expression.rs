use super::{
    Typechecker,
    context::TypecheckerContext,
    error::TypecheckerError,
    r#type::{Type, kind::TypeKind},
};
use crate::ast::node::expression::{
    BinaryComparison, BinaryOperation, BooleanLiteral, FunctionCall, IdentifierReference, IntegerLiteral, StringLiteral,
};

pub trait ExpressionTypecheck {
    fn resolve(
        &mut self,
        context: &mut TypecheckerContext,
        expected_type: Option<&Type>,
    ) -> Result<Type, TypecheckerError>;
}

impl ExpressionTypecheck for IntegerLiteral {
    fn resolve(
        &mut self,
        _context: &mut TypecheckerContext,
        expected_type: Option<&Type>,
    ) -> Result<Type, TypecheckerError> {
        let integer_type = match expected_type {
            Some(r#type) if matches!(r#type.kind, TypeKind::Integer(_)) => r#type.clone(),
            _ => Type::new(TypeKind::Integer(32), self.node.location),
        };

        self.expected_type = Some(integer_type.clone());

        Ok(integer_type)
    }
}

impl ExpressionTypecheck for StringLiteral {
    fn resolve(
        &mut self,
        _context: &mut TypecheckerContext,
        _expected_type: Option<&Type>,
    ) -> Result<Type, TypecheckerError> {
        // All string literals are currently references to `i8`.
        let string_type = Type::new(TypeKind::Reference(Box::new(TypeKind::Integer(8))), self.node.location);
        Ok(string_type)
    }
}

impl ExpressionTypecheck for IdentifierReference {
    fn resolve(
        &mut self,
        context: &mut TypecheckerContext,
        _expected_type: Option<&Type>,
    ) -> Result<Type, TypecheckerError> {
        let function_scope = match &context.function_scope {
            Some(value) => value,
            None => panic!("Identifier reference outside of function scope?"),
        };

        let variable_type = function_scope
            .variables
            .get(&self.name)
            .ok_or(TypecheckerError::undefined_variable(
                self.name.clone(),
                self.node.location,
            ))
            .cloned()?;

        self.expected_type = Some(variable_type.clone());
        Ok(variable_type)
    }
}

impl ExpressionTypecheck for BinaryOperation {
    fn resolve(
        &mut self,
        context: &mut TypecheckerContext,
        expected_type: Option<&Type>,
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
                right_type.location,
            ));
        }

        self.expected_type = Some(left_type.clone());
        Ok(left_type)
    }
}

impl ExpressionTypecheck for FunctionCall {
    fn resolve(
        &mut self,
        context: &mut TypecheckerContext,
        _expected_type: Option<&Type>,
    ) -> Result<Type, TypecheckerError> {
        // To call a function, we need to know its return type.
        let return_type = context
            .functions
            .get(&self.name)
            .ok_or(TypecheckerError::undefined_function(
                self.name.clone(),
                self.node.location,
            ))?
            .clone();

        // We also must typecheck the function call's arguments.
        for argument in &mut self.arguments {
            // TODO: Pass a function parameter's expected type.
            Typechecker::check_expression(argument, context, None)?;
        }

        self.expected_type = Some(return_type.clone());
        Ok(return_type.clone())
    }
}

impl ExpressionTypecheck for BinaryComparison {
    fn resolve(
        &mut self,
        context: &mut TypecheckerContext,
        expected_type: Option<&Type>,
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
                right_type.location,
            ));
        }

        // Both of the types must be an integer.
        if let TypeKind::Integer(_) = left_type.kind {
            Ok(Type::new(TypeKind::Boolean, self.node.location))
        } else {
            Err(TypecheckerError::mismatched_type(
                TypeKind::Integer(32),
                left_type.kind,
                left_type.location,
            ))
        }
    }
}

impl ExpressionTypecheck for BooleanLiteral {
    fn resolve(
        &mut self,
        _context: &mut TypecheckerContext,
        _expected_type: Option<&Type>,
    ) -> Result<Type, TypecheckerError> {
        Ok(Type::new(TypeKind::Boolean, self.node.location))
    }
}
