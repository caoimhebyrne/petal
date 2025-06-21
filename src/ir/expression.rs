use crate::{
    ast::{self, node::expression::IdentifierReference},
    ir::{IntegerLiteral, Value, VariableReference, context::Context, generator::IRResult},
};

/// Visits an expression inthe AST, converting it to a [Value].
pub(crate) trait ExpressionVisitor {
    fn visit(&self, context: &mut Context) -> IRResult<Value>;
}

impl ExpressionVisitor for ast::node::expression::IntegerLiteral {
    fn visit(&self, _context: &mut Context) -> IRResult<Value> {
        Ok(Value::IntegerLiteral(IntegerLiteral {
            value: self.value.try_into().unwrap(),
        }))
    }
}

impl ExpressionVisitor for IdentifierReference {
    fn visit(&self, context: &mut Context) -> IRResult<Value> {
        let variable_index = context.function_scope(self.node)?.find_variable_index(&self.name);
        Ok(Value::VariableReference(VariableReference { variable_index }))
    }
}
