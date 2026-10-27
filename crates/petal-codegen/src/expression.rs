use cranelift::codegen::ir::{InstBuilder, Value, types};
use petal_ast::Expression;

use crate::function::FunctionCodegen;

impl FunctionCodegen<'_> {
    pub(crate) fn visit_expression(&mut self, expression: &Expression) -> Value {
        match expression {
            Expression::IntegerLiteral { value, .. } => self.builder.ins().iconst(types::I64, *value),
        }
    }
}
