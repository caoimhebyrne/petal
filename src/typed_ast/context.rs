use std::collections::BTreeMap;

use crate::{
    core::span::Span,
    module_registry::ModuleId,
    typed_ast::{
        FunctionParameter,
        Statement,
        r#type::TypeId,
    },
};

/// The context associated with the entire type-checking process.
#[derive(Default, Debug)]
pub struct TypeResolverContext {
    /// The generic functions discovered during the first resolving pass.
    generic_functions: BTreeMap<GenericFunctionKey, GenericFunction>,
}

impl TypeResolverContext {
    /// Finds a [`GenericFunction`] given its name.
    pub fn find_generic_function(&self, name: &str) -> Option<(&GenericFunctionKey, &GenericFunction)> {
        self.generic_functions.iter().find(|(_, it)| it.name == name)
    }

    /// Inserts a [`GenericFunction`] into this [`TypeResolverContext`].
    pub fn insert_generic_function(
        &mut self,
        module_id: ModuleId,
        generic_function: GenericFunction,
    ) -> GenericFunctionKey {
        let key = GenericFunctionKey { id: self.generic_functions.len(), module_id };

        // TODO: What should we do if a generic function already exists with (basically) the exact same information?
        self.generic_functions.insert(key, generic_function);

        key
    }
}

/// A primary key of a [`GenericFunction`]. This is a combination of the function ID and the ID of the module that
/// it was defined in.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct GenericFunctionKey {
    /// The ID of the function.
    id: usize,

    /// The ID of the module that the function was defined in.
    pub module_id: ModuleId,
}

/// A generic type parameter of a [`GenericFunction`].
#[derive(Debug, Clone)]
pub struct GenericTypeParameter {
    /// The name of this generic type.
    pub name: String,

    /// The type ID allocated to this generic type.
    pub type_id: TypeId,
}

/// A generic function that should not be included in the final program.
#[derive(Debug)]
pub struct GenericFunction {
    /// The name (as defined in the source code) of this function.
    pub name: String,

    /// The generic type parameters of this function (e.g. `T`).
    pub generic_type_parameters: Vec<GenericTypeParameter>,

    /// The parameters of this function.
    pub parameters: Vec<FunctionParameter>,

    /// The body of this function.
    pub body: Vec<Statement>,

    /// The return type of this function.
    pub return_type_id: TypeId,

    /// The span that this function was defined at in the source code.
    pub span: Span,
}
