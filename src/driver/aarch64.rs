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

pub struct Aarch64Driver {
    output_path: PathBuf,
}

impl Driver for Aarch64Driver {
    fn new(output_path: PathBuf) -> Self {
        Self { output_path }
    }

    fn compile(&self, ir: Vec<Function>) {
        let mut code = String::new();

        for function in ir {
            self.compile_function(&function, &mut code);
        }

        let assembly_file = self.output_path.with_extension("s");
        let output_path = self.output_path.with_extension("o");
        fs::write(&assembly_file, code).unwrap();

        let compile_output = Command::new("as")
            .args([assembly_file.to_str().unwrap(), "-o", output_path.to_str().unwrap()])
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
    fn visit(&self, driver: &Aarch64Driver, function: &Function, code: &mut String);
}

impl OperationVisitor for Store {
    fn visit(&self, driver: &Aarch64Driver, function: &Function, code: &mut String) {
        let variable = function.variables.get(self.variable_index).unwrap();
        let value = driver.compile_value(&self.value, function, code);

        code.push_str(&format!("    mov w8, {}\n", value));
        code.push_str(&format!("    str w8, [sp, {}]\n", variable.stack_index));
    }
}

impl OperationVisitor for Return {
    fn visit(&self, driver: &Aarch64Driver, function: &Function, code: &mut String) {
        if let Some(value) = &self.value {
            let value = driver.compile_value(value, function, code);
            code.push_str(&format!("    ldr w0, {}\n", value));
        }
    }
}

trait ValueVisitor {
    fn visit(&self, driver: &Aarch64Driver, function: &Function, code: &mut String) -> String;
}

impl ValueVisitor for IntegerLiteral {
    fn visit(&self, _driver: &Aarch64Driver, _function: &Function, _code: &mut String) -> String {
        return self.value.to_string();
    }
}

impl ValueVisitor for VariableReference {
    fn visit(&self, _driver: &Aarch64Driver, function: &Function, _code: &mut String) -> String {
        let variable = function.variables.get(self.variable_index).unwrap();
        return format!("[sp, {}]", variable.stack_index);
    }
}
