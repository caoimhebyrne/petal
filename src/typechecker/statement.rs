use super::{Typechecker, context::TypecheckerContext, error::TypecheckerError};
use crate::ast::node::kind::{FunctionDefinitionNode, ReturnNode, VariableDeclarationNode};

pub trait StatmentTypecheck {
    fn resolve<'a>(&mut self, context: &mut TypecheckerContext) -> Result<(), TypecheckerError>;
}

impl StatmentTypecheck for VariableDeclarationNode {
    fn resolve<'a>(&mut self, context: &mut TypecheckerContext) -> Result<(), TypecheckerError> {
        // Before checking the value's type, we must first know what the declared type of the variable is.
        self.declared_type = Typechecker::resolve_type(self.declared_type.clone())?;

        // We can now get the value's type, and check if they are equal.
        let value_type =
            Typechecker::check_expression(&mut self.value, context, Some(&self.declared_type))?;

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

impl StatmentTypecheck for FunctionDefinitionNode {
    fn resolve<'a>(&mut self, context: &mut TypecheckerContext) -> Result<(), TypecheckerError> {
        // We must resolve the return type of the function first.
        if let Some(return_type) = &self.return_type {
            self.return_type = Some(Typechecker::resolve_type(return_type.clone())?);
        }

        // Then, we can check the types within the body.
        context.start_function_scope();

        Typechecker::check_block(&mut self.body, context)?;

        context.end_function_scope();

        Ok(())
    }
}

impl StatmentTypecheck for ReturnNode {
    fn resolve<'a>(&mut self, context: &mut TypecheckerContext) -> Result<(), TypecheckerError> {
        let _value_type = self
            .value
            .as_mut()
            .map(|it| Typechecker::check_expression(it, context, None))
            .transpose()?;

        println!("todo: ensure return expression matches expected block return type");

        Ok(())
    }
}
