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
    typechecker::{
        error::{
            TypecheckerError,
            TypecheckerErrorKind,
        },
        r#type::Type,
    },
};

/// The context of a [`Typechecker`].
#[derive(Default)]
pub(crate) struct TypecheckerContext {
    /// The expected return type of the current function.
    pub(crate) expected_return_type: Type,

    // TODO: Function IDs?
    /// The functions that have been validated by this [`Typechecker`] instance.
    pub(crate) functions: HashMap<String, CheckedFunction>,

    /// The variables that have been declared in the current scope.
    pub(crate) variables: HashMap<String, Type>,
}

impl TypecheckerContext {
    /// Attempts to get a [`CheckedFunction`] from this [`Typechecker`] by its name.
    pub(crate) fn get_checked_function(&self, name: &str, span: Span) -> Result<&CheckedFunction, TypecheckerError> {
        self.functions.get(name).ok_or(TypecheckerErrorKind::UndeclaredFunction(name.into()).at(span))
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
        if self.functions.contains_key(&function_declaration.name) {
            return Err(TypecheckerErrorKind::DuplicateFunctionDeclaration(function_declaration.name.clone()).at(span));
        }

        self.functions.insert(
            function_declaration.name.clone(),
            CheckedFunction::new(function_declaration.parameters.clone(), function_declaration.return_type),
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
    /// The parameters to the function.
    pub parameters: Vec<FunctionParameter>,

    /// The return type of the function.
    pub return_type: Type,
}

impl CheckedFunction {
    /// Creates a new [`CheckedFunction`].
    pub fn new(parameters: Vec<FunctionParameter>, return_type: Type) -> Self {
        Self { parameters, return_type }
    }
}
