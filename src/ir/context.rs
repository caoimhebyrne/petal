/// The context of the intermediate representation generator.
pub struct Context {
    /// The current function's context.
    function_scope: Option<FunctionScope>,
}

/// Information used during the compilation of a single function.
pub struct FunctionScope {
    /// The variables defined within the current function's context.
    pub variables: Vec<String>,
}

impl Context {
    pub fn new() -> Self {
        Self { function_scope: None }
    }

    /// Starts a function scope.
    /// If the previous function scope was not ended, this function will panic.
    pub fn start_function_scope(&mut self) {
        if let Some(_) = self.function_scope {
            panic!(
                "Attempt was made to start a function scope, but the previous context was not ended? (make sure you call end_function_scope)"
            )
        }

        self.function_scope = Some(FunctionScope::new())
    }

    /// Ends the current function's context.
    pub fn end_function_scope(&mut self) {
        self.function_scope = None;
    }

    /// Returns the current function scope, panicing if one is not active.
    pub fn function_scope(&mut self) -> &mut FunctionScope {
        self.function_scope
            .as_mut()
            .expect("Expected a function scope, but none was present...")
    }
}

impl FunctionScope {
    pub fn new() -> Self {
        Self { variables: vec![] }
    }

    /// Declares a variable in this function's context, panicing if a variable was already defined
    /// with the provided name.
    ///
    /// Returns the index of the declared variable.
    pub fn declare_variable<'a>(&mut self, name: &'a str) -> usize {
        if let Some(_) = self.variables.iter().find(|it| *it == name) {
            panic!(
                "A variable was already declared in the current function scope with the name '{}'",
                name
            );
        }

        self.variables.push(name.to_string());
        self.variables.len() - 1
    }

    /// Returns the index for a variable by its name, panicking if it does not exist.
    pub fn find_variable_index<'a>(&mut self, name: &'a str) -> usize {
        self.variables
            .iter()
            .position(|it| it == name)
            .expect(&format!("{} was not declared", name))
    }
}
