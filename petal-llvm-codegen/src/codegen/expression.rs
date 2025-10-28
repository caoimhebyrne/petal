use inkwell::values::{BasicValue, BasicValueEnum};
use petal_ast::expression::{Expression, ExpressionKind};
use petal_core::{error::Result, source_span::SourceSpan};

use crate::{LLVMCodegen, codegen::Codegen, error::LLVMCodegenErrorKind};

impl<'ctx> Codegen<'ctx> for Expression {
    fn codegen(&self, codegen: &'ctx LLVMCodegen, _span: SourceSpan) -> Result<BasicValueEnum<'ctx>> {
        match &self.kind {
            ExpressionKind::IntegerLiteral(value) => {
                let value_type = codegen.create_value_type(self.r#type, self.span)?;
                Ok(value_type
                    .into_int_type()
                    .const_int(*value, false)
                    .as_basic_value_enum())
            }

            _ => return LLVMCodegenErrorKind::unable_to_codegen_expression(&self).into(),
        }
    }
}
