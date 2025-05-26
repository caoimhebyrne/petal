use super::{
    Typechecker,
    context::TypecheckerContext,
    error::TypecheckerError,
    r#type::{Type, kind::TypeKind},
};
use crate::ast::node::statement::{FunctionDefinition, If, Return, VariableDeclaration, VariableReassignment};

pub trait StatementTypecheck {
    fn resolve(&mut self, context: &mut TypecheckerContext) -> Result<(), TypecheckerError>;
}

impl StatementTypecheck for VariableDeclaration {
    fn resolve(&mut self, context: &mut TypecheckerContext) -> Result<(), TypecheckerError> {
        // Before checking the value's type, we must first know what the declared type of the variable is.
        self.declared_type = Typechecker::resolve_type(self.declared_type.clone())?;

        // We can now get the value's type, and check if they are equal.
        let value_type = Typechecker::check_expression(&mut self.value, context, Some(&self.declared_type))?;

        if self.declared_type.kind != value_type.kind {
            return Err(TypecheckerError::mismatched_type(
                self.declared_type.kind.clone(),
                value_type.kind,
                value_type.location,
            ));
        }

        let function_scope = match context.function_scope.as_mut() {
            Some(value) => value,
            None => panic!("Variable declaration outside of function scope?"),
        };

        // The types are valid, we can now record this variable declaration in the current function's scope.
        function_scope
            .variables
            .insert(self.name.clone(), self.declared_type.clone());

        Ok(())
    }
}

impl StatementTypecheck for FunctionDefinition {
    fn resolve(&mut self, context: &mut TypecheckerContext) -> Result<(), TypecheckerError> {
        // We must resolve the return type of the function first.
        let return_type = match &self.return_type {
            Some(r#type) => Typechecker::resolve_type(r#type.clone())?,
            None => Type::new(TypeKind::Void, self.node.location),
        };

        // This may be used by the code generator.
        self.return_type = Some(return_type.clone());

        // Then, we can check the types within the body.
        let scope = context.start_function_scope(&self.name, return_type);
        for parameter in &mut self.parameters {
            parameter.expected_type = Typechecker::resolve_type(parameter.expected_type.clone())?;

            scope
                .variables
                .insert(parameter.name.clone(), parameter.expected_type.clone());
        }

        Typechecker::check_block(&mut self.body, context)?;

        context.end_function_scope();

        Ok(())
    }
}

impl StatementTypecheck for Return {
    fn resolve(&mut self, context: &mut TypecheckerContext) -> Result<(), TypecheckerError> {
        let function_scope = match context.function_scope.clone() {
            Some(value) => value,
            None => panic!("Return statement outside of function scope?"),
        };

        let value_type = match self.value.as_mut() {
            Some(value) => Typechecker::check_expression(value, context, Some(&function_scope.return_type))?,
            None => Type::new(TypeKind::Void, self.node.location),
        };

        if value_type.kind != function_scope.return_type.kind {
            return Err(TypecheckerError::mismatched_type(
                function_scope.return_type.kind,
                value_type.kind,
                self.node.location,
            ));
        }

        Ok(())
    }
}

impl StatementTypecheck for VariableReassignment {
    fn resolve(&mut self, context: &mut TypecheckerContext) -> Result<(), TypecheckerError> {
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

        // The type of the variable must match the type of the value.
        let value_type = Typechecker::check_expression(&mut self.value, context, Some(&variable_type))?;

        // If the variable is a reference, and the right-hand side is not a reference, but they are both
        // the same type-kind, it can be allowed.
        if let TypeKind::Reference(referenced) = &variable_type.kind {
            if **referenced == value_type.kind {
                return Ok(());
            }
        }

        if value_type.kind != variable_type.kind {
            return Err(TypecheckerError::mismatched_type(
                variable_type.kind,
                value_type.kind,
                value_type.location,
            ));
        }

        Ok(())
    }
}

impl StatementTypecheck for If {
    fn resolve(&mut self, context: &mut TypecheckerContext) -> Result<(), TypecheckerError> {
        let expression_type = Typechecker::check_expression(&mut self.condition, context, None)?;
        if expression_type.kind != TypeKind::Boolean {
            return Err(TypecheckerError::mismatched_type(
                TypeKind::Boolean,
                expression_type.kind,
                self.node.location,
            ));
        }

        Typechecker::check_block(&mut self.block, context)?;

        Ok(())
    }
}
