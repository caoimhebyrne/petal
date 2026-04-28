use std::mem::take;

use crate::{
    ast::statement::{
        Statement,
        StatementKind,
        function_declaration::{
            FunctionDeclaration,
            FunctionParameter,
        },
        r#if::If,
        r#return::Return,
        variable_assignment::VariableAssignment,
        variable_declaration::VariableDeclaration,
    },
    core::span::Span,
    typechecker::{
        Typechecker,
        error::{
            TypecheckerError,
            TypecheckerErrorKind,
        },
        r#type::Type,
    },
};

impl Typechecker {
    /// Checks and resolves any [`Type`]s referenced in the provided [`Statement`].
    pub(crate) fn check_statement(&mut self, statement: &mut Statement) -> Result<(), TypecheckerError> {
        match &mut statement.kind {
            StatementKind::FunctionDeclaration(function_declaration) => {
                self.check_function_declaration(function_declaration, statement.span)
            }

            StatementKind::VariableDeclaration(variable_declaration) => {
                self.check_variable_declaration(variable_declaration, statement.span)
            }

            StatementKind::Return(r#return) => self.check_return(r#return, statement.span),

            StatementKind::VariableAssignment(variable_assignment) => {
                self.check_variable_assignment(variable_assignment, statement.span)
            }

            StatementKind::If(r#if) => self.check_if(r#if, statement.span),
        }
    }

    /// Checks and resolves any [`Type`]s referenced in the provided [`FunctionDeclaration`].
    fn check_function_declaration(
        &mut self,
        function_declaration: &mut FunctionDeclaration,
        span: Span,
    ) -> Result<(), TypecheckerError> {
        let previous_variables = take(&mut self.variables);

        function_declaration.return_type = function_declaration
            .return_type_expr
            .as_ref()
            .map(|it| Typechecker::resolve_type_from_expr(it, span))
            .transpose()?
            .unwrap_or(Type::Void);

        for parameter in &mut function_declaration.parameters {
            let parameter_type = Typechecker::check_function_parameter(parameter)?;
            self.insert_variable(parameter.name.clone(), parameter_type, parameter.span)?;
        }

        self.insert_checked_function(function_declaration, span)?;

        // Create a copy of the previous expected return type and variables so that we can restore it later.
        let previous_return_type = self.expected_return_type;
        self.expected_return_type = function_declaration.return_type;

        for statement in &mut function_declaration.body {
            self.check_statement(statement)?;
        }

        self.expected_return_type = previous_return_type;
        self.variables = previous_variables;

        Ok(())
    }

    /// Checks and resolves any [`Type`]s referenced in the provided [`FunctionParameter`].
    fn check_function_parameter(function_parameter: &mut FunctionParameter) -> Result<Type, TypecheckerError> {
        let r#type = Typechecker::resolve_type_from_expr(&function_parameter.type_expr, function_parameter.span)?;
        function_parameter.r#type = r#type;
        Ok(r#type)
    }

    /// Checks and resolves any [`Type`]s referenced in the provided [`VariableDeclaration`].
    fn check_variable_declaration(
        &mut self,
        variable_declaration: &mut VariableDeclaration,
        span: Span,
    ) -> Result<(), TypecheckerError> {
        // The type of the variable must be resolved.
        let variable_type = Typechecker::resolve_type_from_expr(&variable_declaration.type_expr, span)?;

        // The initial value for the variable must have a valid type too, and then that type must be equal to the
        // variable type.
        let value_type = self.check_expression(&mut variable_declaration.value)?;
        if variable_type != value_type {
            return Err(TypecheckerErrorKind::IncompatibleVariableDeclarationTypes {
                declared: variable_type,
                value: value_type,
            }
            .at(span));
        }

        variable_declaration.r#type = variable_type;
        self.insert_variable_from_declaration(variable_declaration, span)?;

        Ok(())
    }

    /// Checks and resolves any [`Type`]s referenced in the provided [`VariableAssignment`].
    fn check_variable_assignment(
        &mut self,
        variable_assignment: &mut VariableAssignment,
        span: Span,
    ) -> Result<(), TypecheckerError> {
        // The variable must already be defined.
        let variable_type = self.get_variable(&variable_assignment.name, span).cloned()?;

        // The initial value for the variable must have a valid type too, and then that type must be equal to the
        // variable type.
        let value_type = self.check_expression(&mut variable_assignment.value)?;
        if variable_type != value_type {
            return Err(TypecheckerErrorKind::IncompatibleVariableDeclarationTypes {
                declared: variable_type,
                value: value_type,
            }
            .at(span));
        }

        Ok(())
    }

    /// Checks and resolves any [`Type`]s referenced in the provided [`Return`].
    fn check_return(&mut self, r#return: &mut Return, span: Span) -> Result<(), TypecheckerError> {
        let value_type = r#return.value.as_mut().map(|it| self.check_expression(it)).transpose()?.unwrap_or(Type::Void);

        // The value being returned must have the same return type as the function being parsed.
        if self.expected_return_type != value_type {
            return Err(TypecheckerErrorKind::IncompatibleReturnTypes {
                declared: self.expected_return_type,
                value: value_type,
            }
            .at(span));
        }

        Ok(())
    }

    /// Checks and resolves any [`Type`]s referenced in the provided [`If`].
    fn check_if(&mut self, r#if: &mut If, _span: Span) -> Result<(), TypecheckerError> {
        // The type of the condition must be a boolean.
        let condition_type = self.check_expression(&mut r#if.condition)?;
        if condition_type != Type::Boolean {
            return Err(TypecheckerErrorKind::IncompatibleTypes { expected: Type::Boolean, got: condition_type }
                .at(r#if.condition.span));
        }

        // All of the statements within the block must be valid.
        let previous_variables = take(&mut self.variables);
        self.variables = previous_variables.clone();

        for statement in &mut r#if.block {
            self.check_statement(statement)?;
        }

        self.variables = previous_variables;

        Ok(())
    }
}
