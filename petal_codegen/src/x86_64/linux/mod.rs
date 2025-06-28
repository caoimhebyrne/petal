use crate::{
    Driver,
    error::{DriverError, DriverResult},
    visitor::{OperationVisitor, ValueVisitor},
};
use petal_ir::{
    function::{Function, LocalKind},
    operation::{Operation, OperationKind},
    value::{Value, ValueKind, ValueType},
};
use std::{
    fs,
    io::{Write, stderr, stdout},
    path::PathBuf,
    process::Command,
};

mod visitor;

pub struct X86_64LinuxDriver {
    /// The lines of assembly to output at the end of visiting the function's statements.
    pub assembly: Vec<String>,
}

impl Driver for X86_64LinuxDriver {
    fn new() -> Self {
        X86_64LinuxDriver { assembly: Vec::new() }
    }

    fn generate(&mut self, functions: Vec<Function>, output_path: &PathBuf) -> DriverResult<()> {
        // The x86_64 linux driver uses assembly syntax, anything else is incorrect :)
        self.assembly.push(".intel_syntax noprefix".to_string());

        // That's all of the setup done, we can start generating functions.
        for mut function in functions {
            self.generate_function(&mut function)?;
        }

        let assembly_file_path = output_path.with_extension("s");
        fs::write(&assembly_file_path, self.assembly.join("\n")).map_err(|e| {
            DriverError::unable_to_write(assembly_file_path.to_str().unwrap().to_string(), e.to_string(), None)
        })?;

        let compile_output = Command::new("cc")
            .args([
                "-o",
                output_path.to_str().unwrap(),
                assembly_file_path.to_str().unwrap(),
            ])
            .output()
            .expect("Failed to execute `cc`.");

        if !compile_output.status.success() {
            let _ = stdout().write_all(&compile_output.stdout);
            let _ = stderr().write_all(&compile_output.stderr);

            return Err(DriverError::compilation_failure(None));
        }

        Ok(())
    }
}

impl X86_64LinuxDriver {
    fn generate_function(&mut self, function: &mut Function) -> DriverResult<()> {
        // Each generated function is marked as global.
        self.assembly.push(format!(".global {}", function.name));
        self.assembly.push(format!("{}:", function.name));

        // Prelude
        self.assembly.push("push rbp".to_string());
        self.assembly.push("mov rbp, rsp".to_string());

        // This will be used later to populate the stack sizing instruction.
        let before_body_idx = self.assembly.len();

        for operation in &function.body.clone() {
            self.visit_operation(function, operation)?;
        }

        // After visiting all instructions, we can calculate the required stack size.
        // The stack is the size of the local variables allocated, and it must be 16-byte aligned.
        let stack_size_unaligned = function
            .locals
            .iter()
            .enumerate()
            // Any parameters with an index higher than 6 should always be on the stack.
            // Any non-parameters should always be on the stack.
            .filter(|(_, it)| it.kind == LocalKind::Variable)
            .map(|(_, it)| X86_64LinuxDriver::size_of(it.value_type))
            .sum::<usize>();

        let stack_size = stack_size_unaligned + 15 & !15;
        if stack_size > 0 {
            self.assembly
                .insert(before_body_idx, format!("sub rsp, {}", stack_size));
        }

        // Epilogue
        if stack_size > 0 {
            self.assembly.push(format!("add rsp, {}", stack_size));
        }

        self.assembly.push("pop rbp".to_string());
        self.assembly.push("ret".to_string());

        Ok(())
    }

    fn visit_operation(&mut self, function: &mut Function, operation: &Operation) -> DriverResult<()> {
        match &operation.kind {
            OperationKind::StoreLocal(store_local) => store_local.visit(function, self),
            OperationKind::Return(r#return) => r#return.visit(function, self),

            #[allow(unreachable_patterns)]
            _ => Err(DriverError::unsupported_operation(
                operation.clone(),
                Some(function.location),
            )),
        }
    }

    fn visit_value(&mut self, function: &mut Function, value: &Value) -> DriverResult<String> {
        match &value.kind {
            ValueKind::IntegerLiteral(integer_literal) => integer_literal.visit(function, self),
            ValueKind::LocalReference(local_reference) => local_reference.visit(function, self),
            ValueKind::BinaryOperation(binary_operation) => binary_operation.visit(function, self),
            ValueKind::FunctionCall(function_call) => function_call.visit(function, self),

            #[allow(unreachable_patterns)]
            _ => Err(DriverError::unsupported_value(value.clone(), Some(function.location))),
        }
    }

    fn local_parameter_register(index: usize, value_type: ValueType, is_reference: bool) -> String {
        match index {
            0 => return "rdi".to_string(),
            1 => return "rsi".to_string(),
            2 => return "rdx".to_string(),
            3 => return "rcx".to_string(),
            4 => return "r8".to_string(),
            5 => return "r9".to_string(),
            _ => {
                let stack_position = index - 6;
                let stack_index = X86_64LinuxDriver::size_of(value_type) * stack_position;
                return format!(
                    "qword ptr [rsp+{}]",
                    if is_reference { 16 + stack_index } else { stack_index }
                );
            }
        }
    }

    fn size_of(value_type: ValueType) -> usize {
        match value_type {
            ValueType::Integer { width } => (width / 4).into(),
        }
    }
}
