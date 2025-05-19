use crate::ast::node::kind::IntegerLiteralNode;

use super::{
    error::TypecheckerError,
    r#type::{Type, kind::TypeKind},
};

pub trait ExpressionTypecheck {
    fn resolve<'a>(&mut self, expected_type: Option<&Type>) -> Result<Type, TypecheckerError>;
}

impl ExpressionTypecheck for IntegerLiteralNode {
    fn resolve<'a>(&mut self, expected_type: Option<&Type>) -> Result<Type, TypecheckerError> {
        let integer_type = match expected_type {
            Some(r#type) if matches!(r#type.kind, TypeKind::Integer(_)) => r#type.clone(),
            _ => Type::new(TypeKind::Integer(32), None),
        };

        self.r#type = Some(integer_type.clone());

        Ok(integer_type)
    }
}
