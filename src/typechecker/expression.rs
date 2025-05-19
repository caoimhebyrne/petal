use crate::{
    ast::node::kind::{IdentifierReferenceNode, IntegerLiteralNode},
    core::location::Location,
};

use super::{
    context::TypecheckerContext,
    error::TypecheckerError,
    r#type::{kind::TypeKind, Type},
};

pub trait ExpressionTypecheck {
    fn resolve<'a>(
        &mut self,
        context: &mut TypecheckerContext,
        expected_type: Option<&Type>,
        location: Location,
    ) -> Result<Type, TypecheckerError>;
}

impl ExpressionTypecheck for IntegerLiteralNode {
    fn resolve<'a>(
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
    fn resolve<'a>(
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
