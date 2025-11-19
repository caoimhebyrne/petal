use petal_ast::expression::{BinaryOperation, StructureInitialization};
use petal_core::{
    error::Result,
    source_span::SourceSpan,
    r#type::{ResolvedType, StructureType},
};

use crate::{Typechecker, error::TypecheckerError, typecheck::Typecheck};

impl<'a> Typecheck<'a> for BinaryOperation {
    fn typecheck(
        &mut self,
        typechecker: &mut Typechecker<'a>,
        expected_type: Option<&ResolvedType>,
        span: SourceSpan,
    ) -> Result<ResolvedType> {
        // The types on the left and right of the operation must both be of the same kind. If that is not the case,
        // then the operation is invalid.
        //
        // FIXME: A user-defined type in the future must be allowed to define what each binary operation does!
        let left_type = typechecker.check_expression(&mut self.left, expected_type)?;
        let right_type = typechecker.check_expression(&mut self.right, expected_type)?;

        if left_type != right_type {
            return TypecheckerError::expected_type(left_type, right_type, span).into();
        }

        Ok(left_type)
    }
}

impl<'a> Typecheck<'a> for StructureInitialization {
    fn typecheck(
        &mut self,
        typechecker: &mut Typechecker<'a>,
        expected_type: Option<&ResolvedType>,
        span: SourceSpan,
    ) -> Result<ResolvedType> {
        // There must be an expected type, and that type must be a structure.
        let structure_type = match expected_type {
            Some(ResolvedType::Structure(value)) => value,

            Some(other) => {
                return TypecheckerError::expected_type(ResolvedType::Structure(StructureType {}), *other, span).into();
            }

            _ => return TypecheckerError::unable_to_resolve_type("anonymous struct", span).into(),
        };

        for (_field_name, field_value) in &mut self.fields {
            // TODO: Ensure that each of the fields within the initializer match the structure's definition.
            typechecker.check_expression(field_value, None)?;
        }

        Ok(ResolvedType::Structure(*structure_type))
    }
}
