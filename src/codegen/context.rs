use inkwell::values::{FunctionValue, PointerValue};
use std::collections::HashMap;

#[derive(Debug)]
pub struct CodegenContext<'ctx> {
    pub function_scope: Option<FunctionScope<'ctx>>,
}

impl<'ctx> CodegenContext<'ctx> {
    pub fn new() -> Self {
        Self { function_scope: None }
    }

    pub fn start_function_scope(&mut self, function: FunctionValue<'ctx>) {
        self.function_scope = Some(FunctionScope::new(function));
    }

    pub fn end_function_scope(&mut self) {
        self.function_scope = None;
    }
}

#[derive(Debug)]
pub struct FunctionScope<'ctx> {
    pub function: FunctionValue<'ctx>,
    pub variables: HashMap<String, PointerValue<'ctx>>,
}

impl<'ctx> FunctionScope<'ctx> {
    pub fn new(function: FunctionValue<'ctx>) -> Self {
        Self {
            function,
            variables: HashMap::new(),
        }
    }
}
