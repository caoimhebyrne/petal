use crate::{
    core::location::Location,
    ir::{Variable, error::IRError, generator::IRResult},
};

use super::error::IRErrorKind;

/// The context of the intermediate representation generator.
pub struct Context {
    /// The current function's context.
    function_scope: Option<FunctionScope>,
}

/// Information used during the compilation of a single function.
pub struct FunctionScope {
    /// The variables defined within the current function's context.
    pub variables: Vec<Variable>,
}

impl Context {
    pub fn new() -> Self {
        Self { function_scope: None }
    }

    /// Starts a function scope.
    /// If the previous function scope was not ended, this function will panic.
    pub fn start_function_scope(&mut self) -> IRResult<()> {
        if let Some(_) = self.function_scope {
            return Err(IRError::new(IRErrorKind::UnterminatedFunctionScope, None));
        }

        self.function_scope = Some(FunctionScope::new());
        Ok(())
    }

    /// Ends the current function's context.
    pub fn end_function_scope(&mut self) {
        self.function_scope = None;
    }

    /// Returns the current function scope, panicing if one is not active.
    pub fn function_scope(&mut self, location: Option<Location>) -> IRResult<&mut FunctionScope> {
        self.function_scope
            .as_mut()
            .ok_or(IRError::new(IRErrorKind::ExpectedFunctionScope, location))
    }
}

impl FunctionScope {
    pub fn new() -> Self {
        Self { variables: Vec::new() }
    }

    /// Declares a variable in this function's context, panicing if a variable was already defined
    /// with the provided name.
    ///
    /// Returns the index of the declared variable.
    pub fn declare_variable<'a>(&mut self, name: &'a str, size: usize) -> usize {
        if let Some(_) = self.variables.iter().find(|it| it.name == name) {
            panic!(
                "A variable was already declared in the current function scope with the name '{}'",
                name
            );
        }

        let stack_size = self.variables.iter().map(|it| it.expected_value_size).sum::<usize>() + size;

        self.variables.push(Variable {
            name: name.to_string(),
            expected_value_size: size,
            stack_index: stack_size,
        });

        self.variables.len() - 1
    }

    /// Returns the index for a variable by its name, panicking if it does not exist.
    pub fn find_variable_index<'a>(&mut self, name: &'a str) -> usize {
        self.variables
            .iter()
            .position(|it| it.name == name)
            .expect(&format!("{} was not declared", name))
    }
}
