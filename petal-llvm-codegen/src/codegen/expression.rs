use inkwell::{
    types::BasicTypeEnum,
    values::{BasicValue, BasicValueEnum},
};
use petal_ast::expression::{identifier_reference::IdentifierReference, integer_literal::IntegerLiteral};
use petal_core::{error::Result, source_span::SourceSpan};

use crate::{LLVMCodegen, codegen::ExpressionCodegen, context::scope::VariableKind, error::IntoCodegenResult};

impl<'ctx> ExpressionCodegen<'ctx> for IntegerLiteral {
    fn generate(
        &self,
        _codegen: &mut LLVMCodegen<'ctx>,
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

impl<'ctx> ExpressionCodegen<'ctx> for IdentifierReference {
    fn generate(
        &self,
        codegen: &mut LLVMCodegen<'ctx>,
        r#type: &BasicTypeEnum<'ctx>,
        span: SourceSpan,
    ) -> Result<BasicValueEnum<'ctx>> {
        let identifier = codegen
            .string_intern_pool
            .resolve_reference_or_err(&self.identifier, span)?;

        // A variable must exist with the provided name.
        let variable = codegen
            .context
            .scope_context(span)?
            .get_variable(&self.identifier, span)?;

        let value = match variable.kind {
            VariableKind::Local(pointer) => pointer.as_basic_value_enum(),
            VariableKind::Parameter(value) => value,
        };

        // If the value is a pointer, and the type is also a pointer, we do not need to do anything.
        if value.is_pointer_value() && r#type.is_pointer_type() {
            return Ok(value);
        }

        // If the value is a pointer, and the type is not a pointer, then we need to load the value from the pointer
        if value.is_pointer_value() && !r#type.is_pointer_type() {
            return codegen
                .llvm_builder
                .build_load(*r#type, value.into_pointer_value(), identifier)
                .map(|it| it.as_basic_value_enum())
                .into_codegen_result(span);
        }

        // NOTE: This will probably caues incorrect codegen in certain cases, but the module verifier should pick it up.
        Ok(value)
    }
}
