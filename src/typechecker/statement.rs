use super::{
    Typechecker,
    context::TypecheckerContext,
    error::TypecheckerError,
    r#type::{Type, kind::TypeKind},
};
use crate::{
    ast::node::kind::{FunctionDefinitionNode, ReturnNode, VariableDeclarationNode},
    core::location::Location,
};

pub trait StatmentTypecheck {
    fn resolve<'a>(&mut self, context: &mut TypecheckerContext, location: Location) -> Result<(), TypecheckerError>;
}

impl StatmentTypecheck for VariableDeclarationNode {
    fn resolve<'a>(&mut self, context: &mut TypecheckerContext, _location: Location) -> Result<(), TypecheckerError> {
        // Before checking the value's type, we must first know what the declared type of the variable is.
        self.declared_type = Typechecker::resolve_type(self.declared_type.clone())?;

        // We can now get the value's type, and check if they are equal.
        let value_type = Typechecker::check_expression(&mut self.value, context, Some(&self.declared_type))?;

        if self.declared_type.kind != value_type.kind {
            return Err(TypecheckerError::mismatched_type(
                self.declared_type.kind.clone(),
                value_type.kind,
                Some(self.value.location),
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
    fn resolve<'a>(&mut self, context: &mut TypecheckerContext, location: Location) -> Result<(), TypecheckerError> {
        // We must resolve the return type of the function first.
        let return_type = match &self.return_type {
            Some(r#type) => Typechecker::resolve_type(r#type.clone())?,
            None => Type::new(TypeKind::Void, Some(location)),
        };

        // This may be used by the code generator.
        self.return_type = Some(return_type.clone());

        // Then, we can check the types within the body.
        context.start_function_scope(return_type);

        Typechecker::check_block(&mut self.body, context)?;

        context.end_function_scope();

        Ok(())
    }
}

impl StatmentTypecheck for ReturnNode {
    fn resolve<'a>(&mut self, context: &mut TypecheckerContext, location: Location) -> Result<(), TypecheckerError> {
        let function_scope = match context.function_scope.clone() {
            Some(value) => value,
            None => panic!("Return statement outside of function scope?"),
        };

        let (value_type, value_location) = match self.value.as_mut() {
            Some(value) => (
                Typechecker::check_expression(value, context, Some(&function_scope.return_type))?,
                value.location,
            ),

            None => (Type::new(TypeKind::Void, Some(location)), location),
        };

        if value_type.kind != function_scope.return_type.kind {
            return Err(TypecheckerError::mismatched_type(
                function_scope.return_type.kind,
                value_type.kind,
                Some(value_location),
            ));
        }

        Ok(())
    }
}
