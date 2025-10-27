use petal_ast::r#type::Type;

#[derive(Debug)]
pub struct TypecheckerContext {
    /// The return type of the current function.
    pub return_type: Type,
}

impl TypecheckerContext {
    pub fn new(return_type: Type) -> Self {
        TypecheckerContext { return_type }
    }
}
