use petal_ast::expression::binary_operation::BinaryOperation;
use petal_core::{error::Result, source_span::SourceSpan, r#type::ResolvedType};

use crate::{Typechecker, error::TypecheckerError, typecheck::Typecheck};

impl<'a> Typecheck<'a> for BinaryOperation {
    fn typecheck(
        &mut self,
        typechecker: &mut Typechecker<'a>,
        expected_type: Option<&ResolvedType>,
        _span: SourceSpan,
    ) -> Result<ResolvedType> {
        // The types of both the left and the right expression must be resolvable.
        let left_type = typechecker.check_expression(&mut self.left, expected_type)?;
        let right_type = typechecker.check_expression(&mut self.right, expected_type)?;

        if left_type != right_type {
            return TypecheckerError::expected_type(left_type, right_type, self.right.span).into();
        }

        Ok(left_type)
    }
}
