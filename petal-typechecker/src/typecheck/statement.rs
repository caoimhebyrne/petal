use petal_ast::{
    statement::{
        function_declaration::FunctionDeclaration, r#return::ReturnStatement, variable_declaration::VariableDeclaration,
    },
    r#type::Type,
};
use petal_core::{error::Result, source_span::SourceSpan};

use crate::{
    Typechecker,
    context::{Function, Variable},
    error::TypecheckerError,
    typecheck::Typecheck,
};

impl<'a> Typecheck<'a> for FunctionDeclaration {
    fn typecheck(&mut self, typechecker: &mut Typechecker<'a>, span: SourceSpan) -> Result<Type> {
        // Before we can type-check anything else, we need to ensure that the function's information (return type,
        // parameters, etc.) is valid.
        self.return_type = typechecker.resolve_type(&self.return_type)?;

        for parameter in &mut self.parameters {
            parameter.value_type = typechecker.resolve_type(&self.return_type)?;
        }

        // Declaring the function will also check if another function exists with the same name. If one does, then an
        // error will be returned.
        typechecker
            .context
            .add_function(&self.name_reference, Function::new(self.return_type, span))?;

        // Now that we've type-checked the function's information, we can create a function context and typecheck any
        // statements within the function body.
        typechecker.context.start_function_context(self.return_type, span)?;

        for statement in &mut self.body {
            typechecker.check_statement(statement)?;
        }

        typechecker.context.end_function_context(span)?;

        // This statement does not have a return value, so we return void instead.
        Ok(Type::void(span))
    }
}

impl<'a> Typecheck<'a> for ReturnStatement {
    fn typecheck(&mut self, typechecker: &mut Typechecker<'a>, span: SourceSpan) -> Result<Type> {
        // A return statement may not have a value. If no value is present, then the "expected type" of the return
        // statement is void.
        let value_type = self
            .value
            .as_mut()
            .map(|it| typechecker.check_expression(it))
            .transpose()?
            .unwrap_or(Type::void(span));

        // If the value's type is not equal to the function's return type, then we must throw an error.
        let function_context = typechecker.context.function_context(span)?;
        if value_type.kind != function_context.return_type.kind {
            return TypecheckerError::expected_type(function_context.return_type.kind, value_type.kind, span).into();
        }

        // This statement does not have a return value, so we return void instead.
        Ok(Type::void(span))
    }
}

impl<'a> Typecheck<'a> for VariableDeclaration {
    fn typecheck(&mut self, typechecker: &mut Typechecker<'a>, span: SourceSpan) -> Result<Type> {
        self.r#type = typechecker.resolve_type(&self.r#type)?;

        // The initial value's type must be the same as the variable's type.
        let value_type = typechecker.check_expression(&mut self.value)?;
        if value_type.kind != self.r#type.kind {
            return TypecheckerError::expected_type(self.r#type.kind, value_type.kind, span).into();
        }

        // We can then insert the variable into the current function context.
        let function_context = typechecker.context.function_context(span)?;
        function_context.add_variable(&self.identifier_reference, Variable::new(self.r#type, span))?;

        // This statement does not have a return value, so we return void instead.
        Ok(Type::void(span))
    }
}
