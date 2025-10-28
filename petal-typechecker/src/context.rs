use std::collections::HashMap;

use petal_ast::r#type::Type;
use petal_core::string_intern::StringReference;

#[derive(Debug)]
pub struct TypecheckerContext {
    /// The return type of the current function.
    pub return_type: Type,

    /// A map of identifier references to their expected type.
    pub variable_declarations: HashMap<StringReference, Type>,
}

impl TypecheckerContext {
    pub fn new(return_type: Type) -> Self {
        TypecheckerContext {
            return_type,
            variable_declarations: HashMap::new(),
        }
    }

    pub fn add_variable_declaration(&mut self, reference: StringReference, r#type: Type) {
        self.variable_declarations.insert(reference, r#type);
    }

    pub fn variable_declaration_exists(&self, reference: StringReference) -> bool {
        self.variable_declarations.contains_key(&reference)
    }

    pub fn get_variable_type(&self, reference: StringReference) -> Option<&Type> {
        self.variable_declarations.get(&reference)
    }
}
