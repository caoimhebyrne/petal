use super::{Typechecker, error::TypecheckerError};
use crate::ast::node::kind::{FunctionDefinitionNode, ReturnNode, VariableDeclarationNode};

pub trait StatmentTypecheck {
    fn resolve<'a>(&mut self) -> Result<(), TypecheckerError>;
}

impl StatmentTypecheck for VariableDeclarationNode {
    fn resolve<'a>(&mut self) -> Result<(), TypecheckerError> {
        // Before checking the value's type, we must first know what the declared type of the variable is.
        self.declared_type = Typechecker::resolve_type(self.declared_type.clone())?;

        // We can now get the value's type, and check if they are equal.
        let value_type = Typechecker::check_expression(&mut self.value, Some(&self.declared_type))?;

        if self.declared_type.kind != value_type.kind {
            return Err(TypecheckerError::mismatched_type(
                self.declared_type.kind.clone(),
                value_type.kind,
                value_type.location,
            ));
        }

        Ok(())
    }
}

impl StatmentTypecheck for FunctionDefinitionNode {
    fn resolve<'a>(&mut self) -> Result<(), TypecheckerError> {
        // We must resolve the return type of the function first.
        if let Some(return_type) = &self.return_type {
            self.return_type = Some(Typechecker::resolve_type(return_type.clone())?);
        }

        // Then, we can check the types within the body.
        Typechecker::check_block(&mut self.body)?;

        Ok(())
    }
}

impl StatmentTypecheck for ReturnNode {
    fn resolve<'a>(&mut self) -> Result<(), TypecheckerError> {
        let _value_type = self
            .value
            .as_mut()
            .map(|it| Typechecker::check_expression(it, None))
            .transpose()?;

        println!("todo: ensure return expression matches expected block return type");

        Ok(())
    }
}
