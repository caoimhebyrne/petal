use std::collections::HashMap;

use petal_core::{
    error::Result,
    source_span::SourceSpan,
    string_intern::{StringInternPool, StringReference},
    r#type::Type,
};

use crate::error::TypecheckerError;

/// The context attached to a [crate::Typechecker].
pub struct TypecheckerContext<'a> {
    /// The [FunctionContext] that is currently bound to this [TypecheckerContext].
    function_context: Option<FunctionContext<'a>>,

    /// A map of [StringReference]s for function names to their [Function]s.
    functions: HashMap<StringReference, Function>,

    /// The [StringInternPool] to read strings from.
    string_intern_pool: &'a dyn StringInternPool,
}

impl<'a> TypecheckerContext<'a> {
    /// Creates a new [TypecheckerContext].
    pub fn new(string_intern_pool: &'a dyn StringInternPool) -> Self {
        TypecheckerContext {
            function_context: None,
            functions: HashMap::new(),
            string_intern_pool,
        }
    }

    /// Creates a [FunctionContext] instance and binds it to this [TypecheckerContext].
    pub fn start_function_context(&mut self, return_type: Type, _span: SourceSpan) -> Result<()> {
        // TODO: Do we need to throw an error if a function context is already bound?

        self.function_context = Some(FunctionContext::new(return_type, self.string_intern_pool));
        Ok(())
    }

    /// Un-binds the function context that may or may not be currently bound to this [TypecheckerContext].
    ///
    /// Errors:
    /// - [TypecheckerError::ExpectedFunctionContext] If a function context has not been bound to this
    ///   [TypecheckerContext] yet.
    pub fn end_function_context(&mut self, span: SourceSpan) -> Result<()> {
        // If a function context has not been bound yet, there is nothing to end.
        if self.function_context.is_none() {
            return TypecheckerError::expected_function_context(span).into();
        }

        self.function_context = None;
        Ok(())
    }

    /// Returns the current [FunctionContext] that is bound to this [TypecheckerContext].
    ///
    /// Errors:
    /// - [TypecheckerError::ExpectedFunctionContext] If a function context has not been bound to this
    ///   [TypecheckerContext] yet.
    pub fn function_context(&mut self, span: SourceSpan) -> Result<&mut FunctionContext<'a>> {
        self.function_context
            .as_mut()
            .ok_or(TypecheckerError::expected_function_context(span))
    }

    /// Adds a [Function] to this [TypecheckerContext].
    ///
    /// Errors:
    /// - [TypecheckerError::DuplicateFunctionDeclaration] If a function has already been declared with the provided
    ///   name.
    pub fn add_function(&mut self, name: &StringReference, function: Function) -> Result<()> {
        if self.functions.get(name).is_some() {
            let function_name = self.string_intern_pool.resolve_reference_or_err(name, function.span)?;
            return TypecheckerError::duplicate_function_declaration(function_name, function.span).into();
        }

        self.functions.insert(*name, function);
        Ok(())
    }

    /// Finds a [Function] within this [TypecheckerContext].
    ///
    /// Errors:
    /// - [TypecheckerError::UndeclaredFunction] If a function with the provided name has not yet been defined.
    pub fn get_function(&mut self, name: &StringReference, span: SourceSpan) -> Result<&Function> {
        self.functions.get(name).ok_or_else(|| {
            let function_name = match self.string_intern_pool.resolve_reference_or_err(name, span) {
                Ok(value) => value,
                Err(error) => return error,
            };

            TypecheckerError::undeclared_function(function_name, span).into()
        })
    }
}

/// A function that has been declared during typechecking.
#[derive(Debug, Clone)]
pub struct Function {
    /// The expected return type of the function.
    pub return_type: Type,

    /// The types of the parameters to the function.
    pub parameters: Vec<Type>,

    /// The span within the source code that this function was declared at.
    pub span: SourceSpan,
}

impl Function {
    /// Creates a new [Function].
    pub fn new(return_type: Type, parameters: Vec<Type>, span: SourceSpan) -> Self {
        Function {
            return_type,
            parameters,
            span,
        }
    }
}

/// A variable that has been declared within a function during typechecking.
#[derive(Debug, Copy, Clone)]
pub struct Variable {
    /// The value type of the variable.
    pub r#type: Type,

    /// The kind of variable that this is.
    #[allow(dead_code)]
    pub kind: VariableKind,

    /// The span within the source code that this variable was declared at.
    pub span: SourceSpan,
}

#[derive(Debug, Copy, Clone)]
pub enum VariableKind {
    /// A variable defined by the user in the block.
    Normal,

    /// A variable defined as a parameter to the block.
    Parameter,
}

impl Variable {
    pub fn new(r#type: Type, kind: VariableKind, span: SourceSpan) -> Self {
        Variable { r#type, kind, span }
    }
}

/// The context information associated with a function during its typechecking.
/// This is not the same as a [Function], consider a [Function] the product of a [FunctionContext].
pub struct FunctionContext<'a> {
    /// The return type of the current function.
    pub return_type: Type,

    /// The [StringInternPool] to read strings from.
    string_intern_pool: &'a dyn StringInternPool,

    /// A map of [StringReference]s for variable names to their [Variable]s.
    variables: HashMap<StringReference, Variable>,
}

impl<'a> FunctionContext<'a> {
    /// Creates a new [FunctionContext].
    pub fn new(return_type: Type, string_intern_pool: &'a dyn StringInternPool) -> Self {
        FunctionContext {
            return_type,
            string_intern_pool,
            variables: HashMap::new(),
        }
    }

    /// Adds a variable to this [FunctionContext].
    ///
    /// Errors:
    /// - [TypecheckerError::DuplicateVariableDeclaration] If a variable has already been declared with the provided
    ///   name.
    pub fn add_variable(&mut self, name: &StringReference, variable: Variable) -> Result<()> {
        if self.variables.get(name).is_some() {
            let variable_name = self.string_intern_pool.resolve_reference_or_err(name, variable.span)?;
            return TypecheckerError::duplicate_variable_declaration(variable_name, variable.span).into();
        }

        self.variables.insert(*name, variable);
        Ok(())
    }

    /// Retrieves a variable by its name from this [FunctionContext].
    ///
    /// Errors:
    /// - [TypecheckerError::UndeclaredVariable] If a variable with the provided name has not yet been declared.
    pub fn get_variable(&self, name: &StringReference, span: SourceSpan) -> Result<&Variable> {
        self.variables.get(name).ok_or_else(|| {
            let function_name = match self.string_intern_pool.resolve_reference_or_err(name, span) {
                Ok(value) => value,
                Err(error) => return error,
            };

            TypecheckerError::undeclared_variable(function_name, span).into()
        })
    }
}
