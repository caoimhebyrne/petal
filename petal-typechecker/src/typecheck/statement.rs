use petal_ast::statement::{
    StatementKind, function_declaration::FunctionDeclaration, r#return::ReturnStatement,
    variable_assignment::VariableAssignment, variable_declaration::VariableDeclaration,
};
use petal_core::{error::Result, source_span::SourceSpan, r#type::ResolvedType};

use crate::{
    Typechecker,
    context::{Function, Variable, VariableKind},
    error::TypecheckerError,
    typecheck::Typecheck,
};

impl<'a> Typecheck<'a> for FunctionDeclaration {
    fn typecheck(&mut self, typechecker: &mut Typechecker<'a>, span: SourceSpan) -> Result<ResolvedType> {
        // Before we can type-check anything else, we need to ensure that the function's information (return type,
        // parameters, etc.) is valid.
        let return_type = typechecker.resolve_type(&self.return_type)?;

        // Now that we've type-checked the function's information, we can create a function context and typecheck any
        // statements within the function body.
        typechecker.context.start_function_context(return_type, span)?;

        let mut parameter_types = Vec::new();

        for parameter in &mut self.parameters {
            let parameter_type = typechecker.resolve_type(&parameter.value_type)?;
            parameter_types.push(parameter_type);

            typechecker.context.function_context(span)?.add_variable(
                &parameter.name_reference,
                Variable::new(parameter_type, VariableKind::Parameter, parameter.span),
            )?;
        }

        // If we do not encounter a return statement in a void method, then we can insert one implicitly.
        let mut found_return_statement = self.is_extern;

        for statement in &mut self.body {
            typechecker.check_statement(statement)?;

            if let StatementKind::ReturnStatement(_) = statement.kind {
                found_return_statement = true;
            }
        }

        if !found_return_statement {
            // If a return statement was not present, and this was a void function, we can insert one implicitly.
            // Otherwise, we must return an error. All blocks must have a terminator and we cannot provide one
            // implicitly for a non-void return type.
            if return_type == ResolvedType::Void {
                self.insert_implicit_return_void(span);
            } else {
                return TypecheckerError::missing_return_statement(span).into();
            }
        }

        typechecker.context.end_function_context(span)?;

        // Declaring the function will also check if another function exists with the same name. If one does, then an
        // error will be returned.
        typechecker
            .context
            .add_function(&self.name_reference, Function::new(return_type, parameter_types, span))?;

        // This statement does not have a return value, so we return void instead.
        Ok(ResolvedType::Void)
    }
}

impl<'a> Typecheck<'a> for ReturnStatement {
    fn typecheck(&mut self, typechecker: &mut Typechecker<'a>, span: SourceSpan) -> Result<ResolvedType> {
        // A return statement may not have a value. If no value is present, then the "expected type" of the return
        // statement is void.
        let value_type = self
            .value
            .as_mut()
            .map(|it| typechecker.check_expression(it))
            .transpose()?
            .unwrap_or(ResolvedType::Void);

        // If the value's type is not equal to the function's return type, then we must throw an error.
        let function_context = typechecker.context.function_context(span)?;
        if value_type != function_context.return_type {
            return TypecheckerError::expected_type(function_context.return_type, value_type, span).into();
        }

        // This statement does not have a return value, so we return void instead.
        Ok(ResolvedType::Void)
    }
}

impl<'a> Typecheck<'a> for VariableDeclaration {
    fn typecheck(&mut self, typechecker: &mut Typechecker<'a>, span: SourceSpan) -> Result<ResolvedType> {
        let r#type = typechecker.resolve_type(&self.r#type)?;

        // The initial value's type must be the same as the variable's type.
        let value_type = typechecker.check_expression(&mut self.value)?;
        if value_type != r#type {
            return TypecheckerError::expected_type(r#type, value_type, span).into();
        }

        // We can then insert the variable into the current function context.
        let function_context = typechecker.context.function_context(span)?;

        function_context.add_variable(
            &self.identifier_reference,
            Variable::new(r#type, VariableKind::Normal, span),
        )?;

        // This statement does not have a return value, so we return void instead.
        Ok(ResolvedType::Void)
    }
}

impl<'a> Typecheck<'a> for VariableAssignment {
    fn typecheck(&mut self, typechecker: &mut Typechecker<'a>, span: SourceSpan) -> Result<ResolvedType> {
        // A variable must have been declared already.
        let variable = *typechecker
            .context
            .function_context(span)?
            .get_variable(&self.identifier_reference, span)?;

        // If the type of the variable does not match the value type, then this is not possible.
        let value_type = typechecker.check_expression(&mut self.value)?;
        if !value_type.is_assignable_to(&typechecker.type_pool, &variable.r#type, span)? {
            return TypecheckerError::expected_type(variable.r#type, value_type, span).into();
        }

        // This statement does not have a return value, so we return void instead.
        Ok(ResolvedType::Void)
    }
}
