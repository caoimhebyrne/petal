use std::{
    fs,
    io::{Write, stderr, stdout},
    path::PathBuf,
    process::Command,
};

use crate::{
    driver::Driver,
    ir::{Function, Operation, Value},
};

pub struct X86_64Driver {
    output_path: PathBuf,
}

impl Driver for X86_64Driver {
    fn new(output_path: PathBuf) -> Self {
        Self { output_path }
    }

    fn compile(&self, ir: Vec<Function>) {
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

        let assembly_file = self.output_path.with_extension("s");
        let output_path = self.output_path.with_extension("o");
        fs::write(&assembly_file, code).unwrap();

        let compile_output = Command::new("as")
            .args([
                "-g",
                "-mintel64",
                assembly_file.to_str().unwrap(),
                "-o",
                output_path.to_str().unwrap(),
            ])
            .output()
            .expect("Failed to compile for output path");

        stdout().write_all(&compile_output.stdout).unwrap();
        stderr().write_all(&compile_output.stderr).unwrap();

        let link_output = Command::new("ld")
            .args(["-o", self.output_path.to_str().unwrap(), output_path.to_str().unwrap()])
            .output()
            .expect("Failed to compile for output path");

        stdout().write_all(&link_output.stdout).unwrap();
        stderr().write_all(&link_output.stderr).unwrap();
    }
}

impl X86_64Driver {
    fn compile_function(&self, function: Function, code: &mut String) {
        let stack_size: usize = function.variables.iter().map(|it| it.expected_value_size).sum();
        let stack_size_aligned = (stack_size + 15) & !15;

        // This is the start of every function.
        code.push_str(&format!("{}:\n", function.name));

        code.push_str("    push rbp\n");
        code.push_str("    mov rbp, rsp\n");
        code.push_str(&format!("    sub rsp, {}\n", stack_size_aligned));

        // Each function is just a list of operations.
        for operation in &function.body {
            self.compile_operation(operation, &function, code);
        }

        code.push_str(&format!("    add rsp, {}\n", stack_size_aligned));
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
