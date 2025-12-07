use inkwell::{
    types::BasicTypeEnum,
    values::{BasicValue, BasicValueEnum},
};
use petal_ast::expression::{
    binary_operation::{BinaryOperation, BinaryOperationKind},
    boolean_literal::BooleanLiteral,
    identifier_reference::IdentifierReference,
    integer_literal::IntegerLiteral,
    string_literal::StringLiteral,
};
use petal_core::{error::Result, source_span::SourceSpan};

use crate::{
    LLVMCodegen,
    codegen::{ExpressionCodegen, ExpressionCodegenOptions},
    context::scope::VariableKind,
    error::IntoCodegenResult,
};

impl<'ctx> ExpressionCodegen<'ctx> for IntegerLiteral {
    fn generate(
        &self,
        _codegen: &mut LLVMCodegen<'ctx>,
        r#type: &BasicTypeEnum<'ctx>,
        _options: ExpressionCodegenOptions,
        _span: SourceSpan,
    ) -> Result<BasicValueEnum<'ctx>> {
        // The provided basic type **must** be an integer type.
        Ok(r#type
            .into_int_type()
            .const_int(self.value, false)
            .as_basic_value_enum())
    }
}

impl<'ctx> ExpressionCodegen<'ctx> for StringLiteral {
    fn generate(
        &self,
        codegen: &mut LLVMCodegen<'ctx>,
        _type: &BasicTypeEnum<'ctx>,
        _options: ExpressionCodegenOptions,
        span: SourceSpan,
    ) -> Result<BasicValueEnum<'ctx>> {
        let string_value = codegen.string_intern_pool.resolve_reference_or_err(&self.value, span)?;

        codegen
            .llvm_builder
            .build_global_string_ptr(string_value, &format!("string_{}", self.value.0))
            .map(|it| it.as_basic_value_enum())
            .into_codegen_result(span)
    }
}

impl<'ctx> ExpressionCodegen<'ctx> for IdentifierReference {
    fn generate(
        &self,
        codegen: &mut LLVMCodegen<'ctx>,
        _type: &BasicTypeEnum<'ctx>,
        options: ExpressionCodegenOptions,
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

        match variable.kind {
            VariableKind::Local(pointer) => {
                if options.as_reference {
                    Ok(pointer.as_basic_value_enum())
                } else {
                    codegen
                        .llvm_builder
                        .build_load(variable.value_type, pointer, identifier)
                        .into_codegen_result(span)
                }
            }

            VariableKind::Parameter(value) => Ok(value),
        }
    }
}

impl<'ctx> ExpressionCodegen<'ctx> for BinaryOperation {
    fn generate(
        &self,
        codegen: &mut LLVMCodegen<'ctx>,
        _type: &BasicTypeEnum<'ctx>,
        _options: ExpressionCodegenOptions,
        span: SourceSpan,
    ) -> Result<BasicValueEnum<'ctx>> {
        // FIXME: Both values must be an integer.
        let left = codegen.visit_expression(&self.left)?.into_int_value();
        let right = codegen.visit_expression(&self.right)?.into_int_value();

        match self.kind {
            BinaryOperationKind::Add => codegen.llvm_builder.build_int_add(left, right, "add"),
            BinaryOperationKind::Subtract => codegen.llvm_builder.build_int_sub(left, right, "sub"),
            BinaryOperationKind::Divide => codegen.llvm_builder.build_int_signed_div(left, right, "div"),
            BinaryOperationKind::Multiply => codegen.llvm_builder.build_int_mul(left, right, "mul"),
        }
        .into_codegen_result(span)
        .map(|it| it.as_basic_value_enum())
    }
}

impl<'ctx> ExpressionCodegen<'ctx> for BooleanLiteral {
    fn generate(
        &self,
        codegen: &mut LLVMCodegen<'ctx>,
        _type: &BasicTypeEnum<'ctx>,
        _options: ExpressionCodegenOptions,
        _span: SourceSpan,
    ) -> Result<BasicValueEnum<'ctx>> {
        let value = if self.value { 1 } else { 0 };

        Ok(codegen
            .llvm_context
            .bool_type()
            .const_int(value, false)
            .as_basic_value_enum())
    }
}
