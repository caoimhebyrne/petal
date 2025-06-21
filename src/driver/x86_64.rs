use std::{
    fs,
    io::{Write, stderr, stdout},
    path::PathBuf,
    process::Command,
};

use crate::{
    driver::Driver,
    ir::{Function, IntegerLiteral, Operation, Return, Store, Value, VariableReference},
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
                "-mintel64",
                assembly_file.to_str().unwrap(),
                "-o",
                output_path.to_str().unwrap(),
            ])
            .output()
            .expect("Failed to compile for output path");

        if !compile_output.status.success() {
            stdout().write_all(&compile_output.stdout).unwrap();
            stderr().write_all(&compile_output.stderr).unwrap();

            panic!("Failed to assemble, see the logs above for more information.");
        }

        let link_output = Command::new("ld")
            .args(["-o", self.output_path.to_str().unwrap(), output_path.to_str().unwrap()])
            .output()
            .expect("Failed to compile for output path");

        if !link_output.status.success() {
            stdout().write_all(&link_output.stdout).unwrap();
            stderr().write_all(&link_output.stderr).unwrap();

            panic!("Failed to link, see the logs above for more information.");
        }
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
            Operation::Store(store) => store.visit(self, function, code),
            Operation::Return(r#return) => r#return.visit(self, function, code),
        }
    }

    fn compile_value(&self, value: &Value, function: &Function, code: &mut String) -> String {
        match value {
            Value::IntegerLiteral(literal) => literal.visit(self, function, code),
            Value::VariableReference(reference) => reference.visit(self, function, code),
        }
    }
}

trait OperationVisitor {
    fn visit(&self, driver: &X86_64Driver, function: &Function, code: &mut String);
}

impl OperationVisitor for Store {
    fn visit(&self, driver: &X86_64Driver, function: &Function, code: &mut String) {
        let variable = function.variables.get(self.variable_index).unwrap();
        let value = driver.compile_value(&self.value, function, code);

        code.push_str(&format!(
            "    mov dword ptr [rbp-{}], {}\n",
            variable.stack_index, value
        ));
    }
}

impl OperationVisitor for Return {
    fn visit(&self, driver: &X86_64Driver, function: &Function, code: &mut String) {
        if let Some(value) = &self.value {
            let value = driver.compile_value(value, function, code);
            code.push_str(&format!("    mov eax, {}\n", value));
        }
    }
}

trait ValueVisitor {
    fn visit(&self, driver: &X86_64Driver, function: &Function, code: &mut String) -> String;
}

impl ValueVisitor for IntegerLiteral {
    fn visit(&self, _driver: &X86_64Driver, _function: &Function, _code: &mut String) -> String {
        return self.value.to_string();
    }
}

impl ValueVisitor for VariableReference {
    fn visit(&self, _driver: &X86_64Driver, function: &Function, _code: &mut String) -> String {
        let variable = function.variables.get(self.variable_index).unwrap();
        return format!("dword ptr [rbp-{}]", variable.stack_index);
    }
}
