use super::r#type::Type;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct TypecheckerContext {
    pub function_scope: Option<FunctionScope>,
    pub functions: HashMap<String, Type>,
}

impl TypecheckerContext {
    pub fn new() -> Self {
        Self {
            function_scope: None,
            functions: HashMap::new(),
        }
    }

    pub fn start_function_scope(&mut self, name: &str, return_type: Type) {
        self.functions.insert(name.to_owned(), return_type.clone());
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
    pub fn new(return_type: Type) -> Self {
        Self {
            variables: HashMap::new(),
            return_type,
        }
    }
}
