use petal_ast::{
    expression::{Expression, ExpressionKind},
    r#type::{ResolvedTypeKind, Type},
};
use petal_core::{error::Result, source_span::SourceSpan};

use crate::{Typechecker, error::TypecheckerErrorKind, typecheck::Typecheck};

impl Typecheck for Expression {
    fn typecheck(&mut self, typechecker: &mut Typechecker, _span: SourceSpan) -> Result<Type> {
        // If the expression already has a type, then we can just return that.
        if let Some(r#type) = self.r#type {
            return Ok(r#type);
        }

        // Otherwise, we need to create a type.
        //
        // TODO: We need to be able to pass an 'expected type' for the value. For example, when typechecking a return
        //       statement, we should attempt to coerce an integer literal to be the same type as the integer return
        //       type.
        let r#type = match self.kind {
            ExpressionKind::IntegerLiteral(_) => Type::new(ResolvedTypeKind::Integer(32), self.span),

            ExpressionKind::IdentifierReference(reference) => {
                // We take an immutable reference so that it can be passed into the ok_or_else block.
                let string_intern_pool = typechecker.string_intern_pool;

                *typechecker
                    .context(Some(self.span))?
                    .get_variable_type(reference)
                    .ok_or_else(|| {
                        // TODO: error
                        let variable_name = string_intern_pool.resolve_reference(&reference).unwrap();
                        TypecheckerErrorKind::undeclared_variable(variable_name, self.span)
                    })?
            }

            #[allow(unreachable_patterns)]
            _ => return TypecheckerErrorKind::unsupported_expression(&self).into(),
        };

        self.r#type = Some(r#type);
        Ok(r#type)
    }
}
