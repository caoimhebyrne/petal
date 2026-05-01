use std::collections::HashMap;

use crate::{
    ast::statement::{
        function_declaration::{
            FunctionDeclaration,
            FunctionParameter,
        },
        variable_declaration::VariableDeclaration,
    },
    core::span::Span,
    module_registry::ModuleId,
    typechecker::{
        error::{
            TypecheckerError,
            TypecheckerErrorKind,
        },
        r#type::Type,
    },
};

/// A unique identifier for a function.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct FunctionId(usize);

/// The context of a [`Typechecker`].
#[derive(Default)]
pub(crate) struct TypecheckerContext {
    /// The expected return type of the current function.
    pub(crate) expected_return_type: Type,

    /// The functions that have been validated by this [`Typechecker`] instance.
    pub(crate) functions: HashMap<FunctionId, CheckedFunction>,

    /// The variables that have been declared in the current scope.
    pub(crate) variables: HashMap<String, Type>,
}

impl TypecheckerContext {
    /// Attempts to get a [`CheckedFunction`] from this [`Typechecker`] by its name.
    pub(crate) fn get_checked_function(&self, name: &str, span: Span) -> Result<&CheckedFunction, TypecheckerError> {
        // TODO: We should support function overloads.
        // FIXME: Functions need to be generated and prefixed with their module name. How do we do that?
        let function_candidates = self
            .functions
            .values()
            .filter(|it| it.name == name)
            .filter(|it| it.is_visible_to_module(span.module_id))
            .collect::<Vec<_>>();

        if function_candidates.len() > 1 {
            return Err(TypecheckerErrorKind::AmbiguousFunctionCall(name.into()).at(span));
        }

        function_candidates.first().map(|it| *it).ok_or(TypecheckerErrorKind::UndeclaredFunction(name.into()).at(span))
    }

    /// Attempts to get a variable from this [`Typechecker`] by its name.
    pub(crate) fn get_variable(&self, name: &str, span: Span) -> Result<&Type, TypecheckerError> {
        self.variables.get(name).ok_or(TypecheckerErrorKind::UndeclaredVariable(name.into()).at(span))
    }

    /// Inserts a [`CheckedFunction`] into this [`Typechecker`].
    pub(crate) fn insert_checked_function(
        &mut self,
        function_declaration: &FunctionDeclaration,
        span: Span,
    ) -> Result<(), TypecheckerError> {
        let function_id = FunctionId(self.functions.len());

        self.functions.insert(
            function_id,
            CheckedFunction::new(
                span.module_id,
                function_declaration.name.clone(),
                function_declaration.parameters.clone(),
                function_declaration.return_type,
            ),
        );

        Ok(())
    }

    /// Inserts a variable into this [`Typechecker`].
    pub(crate) fn insert_variable_from_declaration(
        &mut self,
        variable_declaration: &VariableDeclaration,
        span: Span,
    ) -> Result<(), TypecheckerError> {
        self.insert_variable(variable_declaration.name.clone(), variable_declaration.r#type, span)
    }

    /// Inserts a variable into this [`Typechecker`].
    pub(crate) fn insert_variable(&mut self, name: String, r#type: Type, span: Span) -> Result<(), TypecheckerError> {
        if self.variables.contains_key(&name) {
            return Err(TypecheckerErrorKind::DuplicateVariableDeclaration(name).at(span));
        }

        self.variables.insert(name, r#type);
        Ok(())
    }
}

/// A function which has been verified by the typechecker.
#[derive(Debug, Clone)]
pub(crate) struct CheckedFunction {
    /// The ID of the module that this function belongs to.
    pub module_id: ModuleId,

    /// The name of the function.
    pub name: String,

    /// The parameters to the function.
    pub parameters: Vec<FunctionParameter>,

    /// The return type of the function.
    pub return_type: Type,
}

impl CheckedFunction {
    /// Creates a new [`CheckedFunction`].
    pub fn new(module_id: ModuleId, name: String, parameters: Vec<FunctionParameter>, return_type: Type) -> Self {
        Self { module_id: module_id, name, parameters, return_type }
    }

    /// Returns whether this [`CheckedFunction`] is visible to the provided module ID.
    ///
    /// By default, all functions are private, and can only be accessed by the module that they are defined in. If a
    /// function is marked with the 'public' modifier, then it can be accessed by any module.
    pub fn is_visible_to_module(&self, other_module_id: ModuleId) -> bool {
        // TODO: Access modifiers
        return self.module_id == other_module_id;
    }
}
