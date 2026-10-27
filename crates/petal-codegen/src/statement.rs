use cranelift::codegen::ir::{InstBuilder, Value};
use petal_ast::{ReturnStatement, Statement};

use crate::function::FunctionCodegen;

impl FunctionCodegen<'_> {
    pub(crate) fn visit_statement(&mut self, statement: &Statement) {
        match statement {
            Statement::Return(statement) => self.visit_return(statement),
        }
    }

    fn visit_return(&mut self, statement: &ReturnStatement) {
        let value: &[Value] = match &statement.value {
            Some(expr) => &[self.visit_expression(expr)],
            None => &[],
        };

        self.builder.ins().return_(value);
    }
}
