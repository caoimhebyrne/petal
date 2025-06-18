use crate::{
    ast::node::expression::{IdentifierReference, IntegerLiteral},
    ir::{Value, context::Context},
};

/// Visits an expression inthe AST, converting it to a [Value].
pub(crate) trait ExpressionVisitor {
    fn visit(&self, context: &mut Context) -> Value;
}

impl ExpressionVisitor for IntegerLiteral {
    fn visit(&self, _context: &mut Context) -> Value {
        Value::IntegerLiteral(self.value.try_into().unwrap())
    }
}

impl ExpressionVisitor for IdentifierReference {
    fn visit(&self, context: &mut Context) -> Value {
        let variable = context.function_scope().find_variable_index(&self.name);
        Value::VariableReference(variable)
    }
}
