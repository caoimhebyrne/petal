use crate::{
    Typechecker,
    context::{Function, Variable, VariableKind},
    error::TypecheckerError,
    typecheck::statement::TypecheckStatement,
};
use petal_ast::statement::{
    StatementNode, StatementNodeKind,
    function_declaration::FunctionDeclaration,
    r#return::Return,
    type_declaration::{TypeDeclaration, TypeDeclarationKind},
};
use petal_core::{
    error::Result,
    source_span::SourceSpan,
    r#type::{ResolvedType, Structure},
};

impl<'a> TypecheckStatement<'a> for FunctionDeclaration {
    fn typecheck_statement(&mut self, typechecker: &mut Typechecker<'a>, span: SourceSpan) -> Result<()> {
        // The return type of the function must be resolvable.
        let return_type = typechecker.resolve_type(&self.return_type)?;

        typechecker.context.start_function_context(return_type.clone(), span)?;

        // The type of each function parameter must be resolvable.
        let parameters = self
            .parameters
            .iter()
            .map(|it| {
                let parameter_type = typechecker.resolve_type(&it.r#type)?;

                typechecker.context.function_context(it.span)?.add_variable(
                    &it.name,
                    Variable::new(parameter_type.clone(), VariableKind::Parameter, it.span),
                )?;

                Ok(parameter_type)
            })
            .collect::<Result<Vec<ResolvedType>>>()?;

        typechecker.context.add_function(
            &self.name,
            Function::new(
                *typechecker.context.module_id(span)?,
                return_type.clone(),
                parameters,
                self.modifiers.clone(),
                span,
            ),
        )?;

        if !self.is_external() {
            // We can then check each statement within the function's body, but we must first add each parameter as a declared variable.
            let mut found_return_statement = false;

            for statement in &mut self.body {
                typechecker.check_statement(statement)?;

                if let StatementNodeKind::Return(_) = statement.kind {
                    found_return_statement = true;
                } else if let StatementNodeKind::If(r#if) = &statement.kind {
                    for then_statement in &r#if.then_block {
                        if let StatementNodeKind::Return(_) = then_statement.kind {
                            found_return_statement = true;
                        }
                    }

                    if found_return_statement {
                        for else_statement in &r#if.else_block {
                            if let StatementNodeKind::Return(_) = else_statement.kind {
                                found_return_statement = true;
                            }
                        }
                    }
                }
            }

            if !found_return_statement {
                if return_type == ResolvedType::Void {
                    let return_statement = StatementNode::new(StatementNodeKind::Return(Return::empty()), span);
                    self.body.push(return_statement);
                } else {
                    return TypecheckerError::missing_return_statement(span).into();
                }
            }
        }

        typechecker.context.end_function_context(span)?;

        Ok(())
    }
}

impl<'a> TypecheckStatement<'a> for TypeDeclaration {
    fn typecheck_statement(&mut self, typechecker: &mut Typechecker<'a>, span: SourceSpan) -> Result<()> {
        match &self.kind {
            TypeDeclarationKind::Structure(_) => {
                // We must first create a structure.
                let structure = Structure::new(self.name);

                // We can then insert it into the type pool.
                let structure_id = typechecker.type_pool.allocate_structure(structure);

                // Then, we can insert a type declaration in the context for this structure.
                // FIXME: cross-module type declarations
                typechecker
                    .context
                    .add_type_declaration(&self.name, ResolvedType::Structure(structure_id), span)
            }
        }
    }
}
