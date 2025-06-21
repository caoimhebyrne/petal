use crate::{
    ast::node::expression::{IdentifierReference, IntegerLiteral},
    ir::{Value, context::Context, generator::IRResult},
};

/// Visits an expression inthe AST, converting it to a [Value].
pub(crate) trait ExpressionVisitor {
    fn visit(&self, context: &mut Context) -> IRResult<Value>;
}

impl ExpressionVisitor for IntegerLiteral {
    fn visit(&self, _context: &mut Context) -> IRResult<Value> {
        Ok(Value::IntegerLiteral(self.value.try_into().unwrap()))
    }
}

impl ExpressionVisitor for IdentifierReference {
    fn visit(&self, context: &mut Context) -> IRResult<Value> {
        let variable = context
            .function_scope(Some(self.node.location))?
            .find_variable_index(&self.name);

        Ok(Value::VariableReference(variable))
    }
}
