use crate::ast::statement::{
    function_declaration::FunctionDeclaration,
    type_declaration::TypeDeclaration,
};

/// The context associated with the entire type-checking process.
#[derive(Default, Debug)]
pub struct TypeResolverContext {
    /// The generic function declarations discovered during the first resolving pass.
    generic_function_declarations: Vec<FunctionDeclaration>,

    /// The generic type declarations discovered during the first resolving pass.
    generic_type_declarations: Vec<TypeDeclaration>,
}

impl TypeResolverContext {
    /// Finds a generic [`FunctionDeclaration`] given its name.
    pub fn find_generic_function_declaration(&self, name: &str) -> Option<&FunctionDeclaration> {
        self.generic_function_declarations.iter().find(|it| it.name == name)
    }

    /// Inserts a generic [`FunctionDeclaration`] into this [`TypeResolverContext`].
    pub fn insert_generic_function_declaration(&mut self, generic_function_declaration: FunctionDeclaration) {
        self.generic_function_declarations.push(generic_function_declaration);
    }

    /// Finds a generic [`TypeDeclaration`] given its name.
    pub fn find_generic_type_declaration(&self, name: &str) -> Option<&TypeDeclaration> {
        self.generic_type_declarations.iter().find(|it| it.name == name)
    }

    /// Inserts a [`TypeDeclaration`] into this [`TypeResolverContext`].
    pub fn insert_generic_type_declaration(&mut self, generic_type_declaration: TypeDeclaration) {
        self.generic_type_declarations.push(generic_type_declaration);
    }
}
