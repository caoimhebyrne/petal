use std::collections::HashMap;

use inkwell::values::PointerValue;

#[derive(Debug)]
pub struct CodegenContext<'ctx> {
    pub function_scope: Option<FunctionScope<'ctx>>,
}

impl<'ctx> CodegenContext<'ctx> {
    pub fn new() -> CodegenContext<'ctx> {
        CodegenContext { function_scope: None }
    }

    pub fn start_function_scope(&mut self) {
        self.function_scope = Some(FunctionScope::new());
    }

    pub fn end_function_scope(&mut self) {
        self.function_scope = None;
    }
}

#[derive(Debug)]
pub struct FunctionScope<'ctx> {
    pub variables: HashMap<String, PointerValue<'ctx>>,
}

impl<'ctx> FunctionScope<'ctx> {
    pub fn new() -> FunctionScope<'ctx> {
        FunctionScope {
            variables: HashMap::new(),
        }
    }
}
