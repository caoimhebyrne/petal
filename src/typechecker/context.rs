use super::r#type::Type;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct TypecheckerContext {
    pub function_scope: Option<FunctionScope>,
}

impl TypecheckerContext {
    pub fn new() -> TypecheckerContext {
        TypecheckerContext {
            function_scope: None,
        }
    }

    pub fn start_function_scope(&mut self, return_type: Type) {
        self.function_scope = Some(FunctionScope::new(return_type));
    }

    pub fn end_function_scope(&mut self) {
        self.function_scope = None;
    }
}

#[derive(Debug, Clone)]
pub struct FunctionScope {
    pub variables: HashMap<String, Type>,
    pub return_type: Type,
}

impl FunctionScope {
    pub fn new(return_type: Type) -> FunctionScope {
        FunctionScope {
            variables: HashMap::new(),
            return_type,
        }
    }
}
