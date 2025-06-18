use crate::ir::{Function, Operation, Value};

// Responsible for generating assembly from the IR.
pub trait Driver {
    fn compile(&self, ir: Vec<Function>) -> String;
}

pub struct X86_64Driver {}

impl Driver for X86_64Driver {
    fn compile(&self, ir: Vec<Function>) -> String {
        let mut code = String::new();
        code.push_str(".intel_syntax noprefix\n");
        code.push_str(".section .text\n");

        for function in ir {
            self.compile_function(function, &mut code);
        }

        code.push_str("\n.global _start\n");
        code.push_str("_start:\n");
        code.push_str("    call main\n");
        code.push_str("    mov edi, eax\n");
        code.push_str("    mov rax, 60\n");
        code.push_str("    syscall\n");

        code
    }
}

impl X86_64Driver {
    fn compile_function(&self, function: Function, code: &mut String) {
        let stack_size: usize = function.variables.iter().map(|it| it.expected_value_size).sum();

        // This is the start of every function.
        code.push_str(&format!("{}:\n", function.name));

        code.push_str("    push rbp\n");
        code.push_str("    mov rbp, rsp\n");
        code.push_str(&format!("    sub rsp, {}\n", stack_size));

        // Each function is just a list of operations.
        for operation in &function.body {
            self.compile_operation(operation, &function, code);
        }

        code.push_str(&format!("    add rsp, {}\n", stack_size));
        code.push_str("    pop rbp\n");
        code.push_str("    ret\n");
    }

    fn compile_operation(&self, operation: &Operation, function: &Function, code: &mut String) {
        match operation {
            Operation::Store { variable_index, value } => {
                let variable = function.variables.get(*variable_index).unwrap();

                code.push_str(&format!(
                    "    mov dword ptr [rbp-{}], {}\n",
                    variable.stack_index,
                    self.compile_value(value, function)
                ));
            }

            Operation::Return { value } => {
                if let Some(the_value) = value {
                    code.push_str(&format!("    mov eax, {}\n", self.compile_value(the_value, function)));
                }
            }
        }
    }

    fn compile_value(&self, value: &Value, function: &Function) -> String {
        match value {
            Value::IntegerLiteral(literal) => literal.to_string(),

            Value::VariableReference(variable_index) => {
                let variable = function.variables.get(*variable_index).unwrap();
                format!("dword ptr [rbp-{}]", variable.stack_index)
            }
        }
    }
}
