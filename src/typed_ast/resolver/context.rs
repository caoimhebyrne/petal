use crate::ast::statement::{
    function_declaration::FunctionDeclaration,
    type_declaration::TypeDeclaration,
};

/// The context associated with the entire type-checking process.
#[derive(Default, Debug)]
pub struct TypeResolverContext {
    /// The function declarations discovered during the first resolving pass.
    function_declarations: Vec<UnresolvedFunctionDeclaration>,

    /// The type declarations discovered during the first resolving pass.
    type_declarations: Vec<TypeDeclaration>,
}

impl TypeResolverContext {
    /// Finds a [`UnresolvedFunctionDeclaration`] given its name.
    pub fn find_function_declaration(&self, name: &str) -> Option<&UnresolvedFunctionDeclaration> {
        self.function_declarations.iter().find(|it| it.declaration.name == name)
    }

    /// Inserts a [`UnresolvedFunctionDeclaration`] into this [`TypeResolverContext`].
    pub fn insert_function_declaration(&mut self, function_declaration: UnresolvedFunctionDeclaration) {
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

/// A function which has yet to be fully visited by the [`TypeResolver`].

#[derive(Debug)]
pub struct UnresolvedFunctionDeclaration {
    /// The namespace that the function was defined in.
    pub namespace: Option<String>,

    /// The original untyped declaration.
    pub declaration: FunctionDeclaration,
}
