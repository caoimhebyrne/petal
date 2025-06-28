use crate::{
    error::{IRError, IRResult},
    function::{Function, Local},
    generator::{
        function_scope::FunctionScope,
        visitor::{expression::ExpressionVisitor, statement::StatementVisitor},
    },
    operation::Operation,
    value::Value,
};
use petal_core::{
    ast::node::{
        expression::Expression,
        statement::{FunctionDefinition, Statement},
    },
    core::location::Location,
};

pub(crate) mod function_scope;
pub(crate) mod value_type;
pub(crate) mod visitor;

/// Responsible for generating an intermediate representation from an abstract syntax tree.
/// This intermediate representation is comprised of [crate::operation]s and [crate::value]s.
pub struct IRGenerator {
    function_scope: Option<FunctionScope>,
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
                function_scope.parameters.push(Local {
                    name: parameter.name.clone(),
                    value_type: parameter.expected_type.clone().into(),
                });
            }
        }

        // We can now parse the function's body.
        let mut body = vec![];
        for statement in &definition.body {
            body.push(self.visit_statement(statement)?);
        }

        // The function's body has been consumed, we can end the function scope.
        let function_scope = self.end_function_scope(definition.node.location)?;

        Ok(Function {
            name: definition.name.clone(),
            location: definition.node.location,
            body,
            locals: function_scope.locals,
            parameters: function_scope.parameters,
        })
    }

    /// Visits a [Statement], returning [Ok] if an implementation of [StatementVisitor] exists for it.
    pub(crate) fn visit_statement(&mut self, statement: &Statement) -> IRResult<Operation> {
        match statement {
            Statement::VariableDeclaration(declaration) => declaration.visit(self),
            Statement::Return(r#return) => r#return.visit(self),

            _ => todo!(),
        }
    }

    /// Visits an [Expression], returning [Ok] if an implementation of [ExpressionVisitor] exists for it.
    pub(crate) fn visit_expression(&mut self, expression: &Expression) -> IRResult<Value> {
        match expression {
            Expression::IntegerLiteral(literal) => literal.visit(self),
            Expression::IdentifierReference(identifier_reference) => identifier_reference.visit(self),
            Expression::BinaryOperation(binary_operation) => binary_operation.visit(self),
            Expression::FunctionCall(function_call) => function_call.visit(self),

            _ => todo!(),
        }
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

        self.function_scope
            .as_mut()
            .ok_or(IRError::expected_function_scope(location))
    }

    /// Attempts to end the current function scope.
    ///
    /// If a function scope was active, it will be returned.
    /// If no function scope is active, an [Err] will be returned.
    pub(crate) fn end_function_scope(&mut self, location: Location) -> IRResult<FunctionScope> {
        let old_scope = match self.function_scope.clone() {
            Some(value) => value,
            _ => return Err(IRError::expected_function_scope(location)),
        };

        self.function_scope = None;

        Ok(old_scope)
    }

    /// Retrieves the current function scope if present.
    /// Returns [Err] if a function scope is not active.
    pub(crate) fn function_scope(&mut self, location: Location) -> IRResult<&mut FunctionScope> {
        self.function_scope
            .as_mut()
            .ok_or(IRError::expected_function_scope(location))
    }
}
