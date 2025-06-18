use crate::{
    driver::Driver,
    ir::{Function, Operation, Value},
};

pub struct Aarch64Driver {}

impl Driver for Aarch64Driver {
    fn compile(&self, ir: Vec<Function>) -> String {
        let mut code = String::new();

        for function in ir {
            self.compile_function(&function, &mut code);
        }

        code
    }
}

impl Aarch64Driver {
    fn compile_function(&self, function: &Function, code: &mut String) {
        code.push_str(&format!(".global {}\n{}:\n", function.name, function.name));

        let stack_size: usize = function.variables.iter().map(|it| it.expected_value_size).sum();
        let stack_size_aligned = (stack_size + 15) & !15;

        code.push_str(&format!("    sub sp, sp, {}\n", stack_size_aligned));

        for operation in &function.body {
            self.compile_operation(&operation, &function, code);
        }

        code.push_str(&format!("    add sp, sp, {}\n", stack_size_aligned));
        code.push_str("    ret\n");
    }

    fn compile_operation(&self, operation: &Operation, function: &Function, code: &mut String) {
        match operation {
            Operation::Store { variable_index, value } => {
                let variable = function.variables.get(*variable_index).unwrap();
                code.push_str(&format!("    mov w8, {}\n", self.compile_value(value, function)));
                code.push_str(&format!("    str w8, [sp, {}]\n", variable.stack_index));
            }

            Operation::Return { value } => {
                if let Some(the_value) = value {
                    code.push_str(&format!("    ldr w0, {}\n", self.compile_value(the_value, function)))
                }
            }
        }
    }

    fn compile_value(&self, value: &Value, function: &Function) -> String {
        match value {
            Value::IntegerLiteral(literal) => literal.to_string(),
            Value::VariableReference(variable_index) => {
                let variable = function.variables.get(*variable_index).unwrap();
                format!("[sp, {}]", variable.stack_index)
            }
        }
    }
}
