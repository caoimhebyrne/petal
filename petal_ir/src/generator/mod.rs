use crate::{
    error::{IRError, IRResult},
    function::{Function, Local},
    generator::function_scope::FunctionScope,
    value::ValueType,
};
use petal_core::{
    ast::node::statement::{FunctionDefinition, Statement},
    core::location::Location,
    typechecker::r#type::kind::TypeKind,
};

pub(crate) mod function_scope;

/// Responsible for generating an intermediate representation from an abstract syntax tree.
/// This intermediate representation is comprised of [crate::operation]s and [crate::value]s.
pub struct IRGenerator {
    pub(crate) function_scope: Option<FunctionScope>,
}

impl IRGenerator {
    /// Creates a new empty IR generator.
    pub fn new() -> IRGenerator {
        IRGenerator { function_scope: None }
    }

    /// Generates IR for a single compilation target.
    ///
    /// The provided [Statement]s must only consist of function definitions. Anything else
    /// is not a supported top-level statement in the intermediate representation, and will
    /// cause an [Err] to be returned.
    pub fn generate(&mut self, statements: Vec<Statement>) -> IRResult<Vec<Function>> {
        let mut functions = vec![];

        for statement in statements {
            if let Statement::FunctionDefinition(definition) = statement {
                functions.push(self.generate_function(&definition)?);
            } else {
                return Err(IRError::unsupported_top_level_statement(statement.node().location));
            }
        }

        Ok(functions)
    }

    /// Generates a [Function] from an AST [FunctionDefinition].
    fn generate_function(&mut self, definition: &FunctionDefinition) -> IRResult<Function> {
        // Before doing anything with the function, we need to ensure that the function scope is started.
        // This allows statement visitors for the function's body to allocate locals, etc.
        {
            // start_function_scope provides a mutable reference to the scope, this allows us to add
            // parameters before parsing the function's body.
            //
            // We must do that in a seperate block within the function, though, as otherwise the mutable
            // reference could live for too long.
            let function_scope = self.start_function_scope(definition.node.location)?;

            for parameter in &definition.parameters {
                let value_type = match &parameter.expected_type.kind {
                    TypeKind::Integer(width) => ValueType::Integer { width: *width },
                    _ => todo!(),
                };

                function_scope.locals.push(Local {
                    name: parameter.name.clone(),
                    value_type,
                });
            }
        }

        // TODO: Parse the function's body.

        // The function's body has been consumed, we can end the function scope.
        let function_scope = self.end_function_scope(definition.node.location)?;

        Ok(Function {
            name: definition.name.clone(),
            location: definition.node.location,
            body: vec![],
            locals: function_scope.locals,
            parameters: function_scope.parameters,
        })
    }

    /// Attempts to start a new function scope.
    ///
    /// If a function scope is already in-progress, this will return an [Err] as nested functions
    /// are not supported at the moment.
    pub(crate) fn start_function_scope(&mut self, location: Location) -> IRResult<&mut FunctionScope> {
        if let Some(_) = self.function_scope {
            return Err(IRError::unterminated_function_scope(location));
        }

        self.function_scope = Some(FunctionScope::new());

        Ok(self
            .function_scope
            .as_mut()
            .expect("Function scope was just set, but as_mut returned None?"))
    }

    /// Attempts to end the current function scope.
    ///
    /// If a function scope was active, it will be returned.
    /// If no function scope is active, an [Err] will be returned.
    pub(crate) fn end_function_scope(&mut self, location: Location) -> IRResult<FunctionScope> {
        let old_scope = match self.function_scope.clone() {
            Some(value) => value,
            _ => return Err(IRError::mismatched_function_scope_termination(location)),
        };

        self.function_scope = None;

        Ok(old_scope)
    }
}
