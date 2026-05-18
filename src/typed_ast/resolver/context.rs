use crate::ast::statement::{
    function_declaration::FunctionDeclaration,
    type_declaration::TypeDeclaration,
};

/// The context associated with the entire type-checking process.
#[derive(Default, Debug)]
pub struct TypeResolverContext {
    /// The function declarations discovered during the first resolving pass.
    function_declarations: Vec<FunctionDeclaration>,

    /// The type declarations discovered during the first resolving pass.
    type_declarations: Vec<TypeDeclaration>,
}

impl TypeResolverContext {
    /// Finds a [`FunctionDeclaration`] given its name.
    pub fn find_function_declaration(&self, name: &str) -> Option<&FunctionDeclaration> {
        self.function_declarations.iter().find(|it| it.name == name)
    }

    /// Inserts a [`FunctionDeclaration`] into this [`TypeResolverContext`].
    pub fn insert_function_declaration(&mut self, function_declaration: FunctionDeclaration) {
        self.function_declarations.push(function_declaration);
    }

    /// Finds a [`TypeDeclaration`] given its name.
    pub fn find_type_declaration(&self, name: &str) -> Option<&TypeDeclaration> {
        self.type_declarations.iter().find(|it| it.name == name)
    }

    /// Inserts a [`TypeDeclaration`] into this [`TypeResolverContext`].
    pub fn insert_type_declaration(&mut self, type_declaration: TypeDeclaration) {
        self.type_declarations.push(type_declaration);
    }
}
