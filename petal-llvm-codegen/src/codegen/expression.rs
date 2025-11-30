use inkwell::values::{BasicValue, BasicValueEnum};
use petal_ast::expression::{
    ExpressionNode, ExpressionNodeKind,
    binary_operation::{BinaryOperation, BinaryOperationKind},
    reference::Reference,
};
use petal_core::{error::Result, source_span::SourceSpan};

use crate::{LLVMCodegen, codegen::Codegen, context::VariableKind, error::LLVMCodegenErrorKind};

impl<'ctx> Codegen<'ctx> for ExpressionNode {
    fn codegen(
        &self,
        codegen: &mut LLVMCodegen<'ctx>,
        span: SourceSpan,
        as_reference: bool,
    ) -> Result<BasicValueEnum<'ctx>> {
        match &self.kind {
            ExpressionNodeKind::IntegerLiteral { value } => {
                let value_type = codegen.resolve_and_create_value_type(self.r#type, self.span)?;

                Ok(value_type
                    .into_int_type()
                    .const_int(*value, false)
                    .as_basic_value_enum())
            }

            ExpressionNodeKind::StringLiteral { value } => {
                let string_value = codegen.string_intern_pool.resolve_reference_or_err(value, span)?;

                let string = codegen
                    .llvm_builder
                    .build_global_string_ptr(string_value, "string")
                    .map_err(|err| LLVMCodegenErrorKind::builder_error(err, span))?;

                Ok(string.as_basic_value_enum())
            }

            ExpressionNodeKind::IdentifierReference { identifier } => {
                let variable = codegen.context.scope_context(span)?.get_variable(identifier, span)?;
                let variable_name = codegen.string_intern_pool.resolve_reference_or_err(identifier, span)?;

                // We have the pointer to the variable, we need to dereference that pointer to get the value.
                let value = match variable.kind {
                    VariableKind::Local(pointer) => {
                        if as_reference {
                            pointer.as_basic_value_enum()
                        } else {
                            codegen
                                .llvm_builder
                                .build_load(variable.value_type, pointer, variable_name)
                                .map_err(|err| LLVMCodegenErrorKind::builder_error(err, span))?
                        }
                    }

                    VariableKind::Parameter(value) => value,
                };

                Ok(value.as_basic_value_enum())
            }

            ExpressionNodeKind::BinaryOperation(binary_operation) => {
                binary_operation.codegen(codegen, span, as_reference)
            }
            ExpressionNodeKind::FunctionCall(call) => call.codegen(codegen, span, as_reference),
            ExpressionNodeKind::Reference(reference) => reference.codegen(codegen, span, as_reference),

            #[allow(unreachable_patterns)]
            _ => return LLVMCodegenErrorKind::unable_to_codegen_expression(&self).into(),
        }
    }
}

impl<'ctx> Codegen<'ctx> for BinaryOperation {
    fn codegen(
        &self,
        codegen: &mut LLVMCodegen<'ctx>,
        span: SourceSpan,
        _as_reference: bool,
    ) -> Result<BasicValueEnum<'ctx>> {
        // FIXME: This assumes that both values are integer types.
        let left_value = self.left.codegen(codegen, span, false)?.into_int_value();
        let right_value = self.right.codegen(codegen, span, false)?.into_int_value();

        let result = match self.kind {
            BinaryOperationKind::Add => codegen.llvm_builder.build_int_add(left_value, right_value, "add"),
            BinaryOperationKind::Subtract => codegen.llvm_builder.build_int_sub(left_value, right_value, "sub"),
            BinaryOperationKind::Multiply => codegen.llvm_builder.build_int_mul(left_value, right_value, "mul"),
            BinaryOperationKind::Divide => codegen
                .llvm_builder
                .build_int_signed_div(left_value, right_value, "div"),
        }
        .map_err(|err| LLVMCodegenErrorKind::builder_error(err, span))?;

        Ok(result.as_basic_value_enum())
    }
}

impl<'ctx> Codegen<'ctx> for Reference {
    fn codegen(
        &self,
        codegen: &mut LLVMCodegen<'ctx>,
        span: SourceSpan,
        _as_reference: bool,
    ) -> Result<BasicValueEnum<'ctx>> {
        let inner_value = self.value.codegen(codegen, self.value.span, true)?;
        if inner_value.is_pointer_value() {
            return Ok(inner_value);
        }

        let local = codegen
            .llvm_builder
            .build_alloca(inner_value.get_type(), "reference")
            .map_err(|err| LLVMCodegenErrorKind::builder_error(err, span))?;

        codegen
            .llvm_builder
            .build_store(local, inner_value)
            .map_err(|err| LLVMCodegenErrorKind::builder_error(err, span))?;

        Ok(local.as_basic_value_enum())
    }
}
