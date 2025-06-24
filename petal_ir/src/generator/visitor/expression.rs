use crate::{
    error::{IRError, IRResult},
    generator::IRGenerator,
    value::Value,
};
use petal_core::ast;

/// A visitor for an AST expression.
/// This converts a [Expression] into an IR [Value].
pub trait ExpressionVisitor {
    fn visit(&self, generator: &mut IRGenerator) -> IRResult<Value>;
}

impl ExpressionVisitor for ast::node::expression::IntegerLiteral {
    fn visit(&self, _generator: &mut IRGenerator) -> IRResult<Value> {
        let value_type = match &self.expected_type {
            Some(value) => value.clone().into(),
            None => return Err(IRError::missing_type_information(self.node.location)),
        };

        Ok(Value::new_integer_literal(self.value, value_type))
    }
}
