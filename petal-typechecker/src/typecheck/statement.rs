use petal_ast::{
    statement::{Statement, StatementKind, function_declaration::FunctionDeclaration, r#return::ReturnStatement},
    r#type::Type,
};
use petal_core::{error::Result, source_span::SourceSpan};

use crate::{Typechecker, context::TypecheckerContext, error::TypecheckerErrorKind, typecheck::Typecheck};

impl Typecheck for Statement {
    fn typecheck(&mut self, typechecker: &mut Typechecker, span: SourceSpan) -> Result<Type> {
        match &mut self.kind {
            StatementKind::FunctionDeclaration(declaration) => declaration.typecheck(typechecker, span),
            StatementKind::ReturnStatement(return_statement) => return_statement.typecheck(typechecker, span),

            _ => return TypecheckerErrorKind::unsupported_statement(self).into(),
        }
    }
}

impl Typecheck for FunctionDeclaration {
    fn typecheck(&mut self, typechecker: &mut Typechecker, _span: SourceSpan) -> Result<Type> {
        let return_type = typechecker.resolve(&mut self.return_type)?;

        // Now that we have resolved the return type, we can bind the function context.
        typechecker.context = Some(TypecheckerContext::new(return_type));

        // We must also type-check each of the statements within the body of the function declaration.
        for statement in &mut self.body {
            statement.typecheck(typechecker, statement.span)?;
        }

        typechecker.context = None;

        Ok(return_type)
    }
}

impl Typecheck for ReturnStatement {
    fn typecheck(&mut self, typechecker: &mut Typechecker, span: SourceSpan) -> Result<Type> {
        let r#type = if let Some(expression) = self.value.as_mut() {
            expression.typecheck(typechecker, expression.span)?
        } else {
            Type::void(span)
        };

        let context = typechecker.context(None)?;
        if context.return_type.kind != r#type.kind {
            return TypecheckerErrorKind::expected_type(&context.return_type, &r#type).into();
        }

        Ok(r#type)
    }
}
