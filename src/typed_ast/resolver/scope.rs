use std::collections::HashMap;

use crate::typed_ast::{
    GenericTypeParameter,
    r#type::db::TypeId,
};

/// The scope of a [`TypeResolver`].
#[derive(Default)]
pub struct Scope {
    /// The generic type parameters that are available in this scope.
    pub generic_type_parameters: Vec<GenericTypeParameter>,

    /// The type of parameters available to this scope.
    parameter_types: HashMap<String, TypeId>,

    /// The parent of this scope, if applicable.
    pub parent: Option<Box<Scope>>,

    /// The type of variables declared within this scope.
    variable_types: HashMap<String, TypeId>,
}

impl Scope {
    /// Creates a scope with parameters and a parent.
    pub fn function(
        generic_type_parameters: Vec<GenericTypeParameter>,
        parameter_types: HashMap<String, TypeId>,
        parent: Option<Self>,
    ) -> Self {
        Self {
            generic_type_parameters,
            parameter_types,
            parent: parent.map(Box::new),
            variable_types: HashMap::default(),
        }
    }

    /// Retrieves the type of an identifier by its name from the current scope.
    ///
    /// If a variable or parameter does not exist with the name in this scope, then the parent scope will be checked
    /// (if present). If one could not be found in any of the parent scopes, then [`None`] will be returned.
    pub fn get_identifier_ty(&self, identifier: &str) -> Option<&TypeId> {
        self.parameter_types
            .get(identifier)
            .or_else(|| self.variable_types.get(identifier))
            .or_else(|| self.parent.as_ref().and_then(|it| it.get_identifier_ty(identifier)))
    }

    /// Retrieves the type of a variable by its name from the current scope.
    ///
    /// If a variable does not exist with the name in this scope, then the parent scope will be checked
    /// (if present). If one could not be found in any of the parent scopes, then [`None`] will be returned.
    pub fn get_variable_ty(&self, variable_name: &str) -> Option<&TypeId> {
        self.variable_types
            .get(variable_name)
            .or_else(|| self.parent.as_ref().and_then(|it| it.get_identifier_ty(variable_name)))
    }

    /// Inserts the type of a variable into the current scope, returning `true` if successful.
    ///
    /// This function will return `false` if a variable exists with the same name in this [`Scope`], or any of its
    /// parent [`Scope`]s.
    pub fn set_variable_ty(&mut self, variable_name: String, type_id: TypeId) -> bool {
        // todo(resolver): what should we do about parameters here?

        if self.get_identifier_ty(&variable_name).is_some() {
            false
        } else {
            self.variable_types.insert(variable_name, type_id);
            true
        }
    }
}
