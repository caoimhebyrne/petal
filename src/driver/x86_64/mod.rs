use crate::{
    driver::{
        Driver, DriverResult,
        error::DriverError,
        x86_64::{operation::OperationVisitor, value::ValueVisitor},
    },
    ir::{Function, Operation, Value},
};
use std::{
    fs,
    io::{Write, stderr, stdout},
    path::PathBuf,
    process::Command,
};

mod operation;
mod value;

pub struct X86_64Driver {
    output_path: PathBuf,
}

impl Driver for X86_64Driver {
    fn new(output_path: PathBuf) -> Self {
        Self { output_path }
    }

    fn compile(&self, ir: Vec<Function>) -> DriverResult<()> {
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
        let assembly_file_str = assembly_file.to_str().expect("Invalid path?");

        let object_path = self.output_path.with_extension("o");
        let object_path_str = object_path.to_str().expect("Invalid path?");

        fs::write(&assembly_file, code).map_err(|_| DriverError::IOError {
            file_name: assembly_file_str.to_owned(),
        })?;

        let compile_output = Command::new("as")
            .args(["-mintel64", assembly_file_str, "-o", object_path_str])
            .output()
            .map_err(|_| DriverError::CompilationFailure)?;

        if !compile_output.status.success() {
            let _ = stdout().write_all(&compile_output.stdout);
            let _ = stderr().write_all(&compile_output.stderr);

            return Err(DriverError::CompilationFailure);
        }

        let link_output = Command::new("ld")
            .args(["-o", self.output_path.to_str().unwrap(), object_path_str])
            .output()
            .map_err(|_| DriverError::LinkingFailure)?;

        if !link_output.status.success() {
            let _ = stdout().write_all(&link_output.stdout);
            let _ = stderr().write_all(&link_output.stderr);

            return Err(DriverError::LinkingFailure);
        }

        Ok(())
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
