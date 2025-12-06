use inkwell::{
    types::BasicTypeEnum,
    values::{BasicValue, BasicValueEnum},
};
use petal_ast::expression::integer_literal::IntegerLiteral;
use petal_core::{error::Result, source_span::SourceSpan};

use crate::{LLVMCodegen, codegen::ExpressionCodegen};

impl<'ctx> ExpressionCodegen<'ctx> for IntegerLiteral {
    fn generate(
        &self,
        _codegen: &mut LLVMCodegen,
        r#type: &BasicTypeEnum<'ctx>,
        _span: SourceSpan,
    ) -> Result<BasicValueEnum<'ctx>> {
        // The provided basic type **must** be an integer type.
        Ok(r#type
            .into_int_type()
            .const_int(self.value, false)
            .as_basic_value_enum())
    }
}
