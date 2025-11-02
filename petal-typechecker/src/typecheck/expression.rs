use petal_ast::{expression::BinaryOperation, r#type::Type};
use petal_core::{error::Result, source_span::SourceSpan};

use crate::{Typechecker, error::TypecheckerError, typecheck::Typecheck};

impl<'a> Typecheck<'a> for BinaryOperation {
    fn typecheck(&mut self, typechecker: &mut Typechecker<'a>, span: SourceSpan) -> Result<Type> {
        // The types on the left and right of the operation must both be of the same kind. If that is not the case,
        // then the operation is invalid.
        //
        // FIXME: A user-defined type in the future must be allowed to define what each binary operation does!
        let left_type = typechecker.check_expression(&mut self.left)?;
        let right_type = typechecker.check_expression(&mut self.right)?;

        if left_type.kind != right_type.kind {
            return TypecheckerError::expected_type(left_type.kind, right_type.kind, span).into();
        }

        Ok(left_type)
    }
}
