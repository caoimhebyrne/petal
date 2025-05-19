use super::r#type::Type;
use std::collections::HashMap;

#[derive(Debug)]
pub struct TypecheckerContext {
    pub function_scope: Option<FunctionScope>,
}

impl TypecheckerContext {
    pub fn new() -> TypecheckerContext {
        TypecheckerContext {
            function_scope: None,
        }
    }

    pub fn start_function_scope(&mut self) {
        self.function_scope = Some(FunctionScope::new());
    }

    pub fn end_function_scope(&mut self) {
        self.function_scope = None;
    }
}

#[derive(Debug)]
pub struct FunctionScope {
    pub variables: HashMap<String, Type>,
}

impl FunctionScope {
    pub fn new() -> FunctionScope {
        FunctionScope {
            variables: HashMap::new(),
        }
    }
}
