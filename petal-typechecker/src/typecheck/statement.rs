use crate::{
    Typechecker,
    context::{Variable, VariableKind},
    error::TypecheckerError,
    typecheck::Typecheck,
};
use petal_ast::statement::{
    r#if::If, r#return::Return, variable_assignment::VariableAssignment, variable_declaration::VariableDeclaration,
};
use petal_core::{error::Result, source_span::SourceSpan, r#type::ResolvedType};

/// An overload of [Typecheck] for statements that does not take an expected type or return a typee.
pub(crate) trait TypecheckStatement<'a> {
    fn typecheck_statement(&mut self, typechecker: &mut Typechecker<'a>, span: SourceSpan) -> Result<()>;
}

impl<'a, T: TypecheckStatement<'a>> Typecheck<'a> for T {
    fn typecheck(
        &mut self,
        typechecker: &mut Typechecker<'a>,
        _expected_type: Option<&ResolvedType>,
        span: SourceSpan,
    ) -> Result<ResolvedType> {
        self.typecheck_statement(typechecker, span)?;
        Ok(ResolvedType::Void)
    }
}

impl<'a> TypecheckStatement<'a> for Return {
    fn typecheck_statement(&mut self, typechecker: &mut Typechecker<'a>, span: SourceSpan) -> Result<()> {
        let return_type = typechecker
            .context
            .function_context(span)
            .map(|it| it.return_type.clone())?;

        // The return type of the value must be resolvable. If there is no value, then it is assumed to be void.
        let value_type = self
            .value
            .as_mut()
            .map(|it| typechecker.check_expression(it, Some(&return_type)))
            .transpose()?
            .unwrap_or(ResolvedType::Void);

        if value_type != return_type {
            return TypecheckerError::expected_type(return_type, value_type, span).into();
        }

        Ok(())
    }
}

impl<'a> TypecheckStatement<'a> for VariableDeclaration {
    fn typecheck_statement(&mut self, typechecker: &mut Typechecker<'a>, span: SourceSpan) -> Result<()> {
        // The type of the variable's value must be resolvable.
        let variable_type = typechecker.resolve_type(&self.r#type)?;

        // The type of the initial value must equal the type of the variable.
        let value_type = typechecker.check_expression(&mut self.value, Some(&variable_type))?;

        if !value_type.is_assignable_to(&typechecker.type_pool, &variable_type, self.value.span)? {
            return TypecheckerError::expected_type(variable_type, value_type, span).into();
        }

        typechecker
            .context
            .function_context(span)?
            .add_variable(&self.name, Variable::new(variable_type, VariableKind::Normal, span))?;

        Ok(())
    }
}

impl<'a> TypecheckStatement<'a> for VariableAssignment {
    fn typecheck_statement(&mut self, typechecker: &mut Typechecker<'a>, span: SourceSpan) -> Result<()> {
        // A variable must exist with the provided name.
        let variable = typechecker
            .context
            .function_context(span)?
            .get_variable(&self.name, span)?
            .clone();

        // The type of the value being assigned to it must equal the variable's type.
        let value_type = typechecker.check_expression(&mut self.value, Some(&variable.r#type))?;
        if !value_type.is_assignable_to(&typechecker.type_pool, &variable.r#type, self.value.span)? {
            return TypecheckerError::expected_type(variable.r#type, value_type, self.value.span).into();
        }

        Ok(())
    }
}

impl<'a> TypecheckStatement<'a> for If {
    fn typecheck_statement(&mut self, typechecker: &mut Typechecker<'a>, span: SourceSpan) -> Result<()> {
        // The condition must be resolvable, and it must be a boolean.
        let condition_type = typechecker.check_expression(&mut self.condition, Some(&ResolvedType::Boolean))?;
        if condition_type != ResolvedType::Boolean {
            return TypecheckerError::expected_type(ResolvedType::Boolean, condition_type, span).into();
        }

        for statement in &mut self.block {
            typechecker.check_statement(statement)?;
        }

        Ok(())
    }
}
