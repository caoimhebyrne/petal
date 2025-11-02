use inkwell::values::{BasicValue, BasicValueEnum};
use petal_ast::expression::{Expression, ExpressionKind};
use petal_core::{error::Result, source_span::SourceSpan};

use crate::{LLVMCodegen, codegen::Codegen, error::LLVMCodegenErrorKind, string_intern_pool_ext::StringInternPoolExt};

impl<'ctx> Codegen<'ctx> for Expression {
    fn codegen(&self, codegen: &mut LLVMCodegen<'ctx>, span: SourceSpan) -> Result<BasicValueEnum<'ctx>> {
        match &self.kind {
            ExpressionKind::IntegerLiteral(value) => {
                let value_type = codegen.create_value_type(self.r#type, self.span)?;
                Ok(value_type
                    .into_int_type()
                    .const_int(*value, false)
                    .as_basic_value_enum())
            }

            ExpressionKind::IdentifierReference(reference) => {
                let variable = codegen.context.scope_context(span)?.get_variable(reference, span)?;
                let variable_name = codegen.string_intern_pool.resolve_reference_or_err(reference, span)?;

                // We have the pointer to the variable, we need to dereference that pointer to get the value.
                let value = codegen
                    .llvm_builder
                    .build_load(variable.value_type, variable.pointer, variable_name)
                    .map_err(|err| LLVMCodegenErrorKind::builder_error(err, span))?;

                Ok(value.as_basic_value_enum())
            }

            #[allow(unreachable_patterns)]
            _ => return LLVMCodegenErrorKind::unable_to_codegen_expression(&self).into(),
        }
    }
}
