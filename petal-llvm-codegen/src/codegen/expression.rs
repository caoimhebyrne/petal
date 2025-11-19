use inkwell::values::{BasicValue, BasicValueEnum};
use petal_ast::expression::{BinaryOperation, Expression, ExpressionKind, Operation, StructureInitialization};
use petal_core::{error::Result, source_span::SourceSpan, r#type::TypeReference};

use crate::{LLVMCodegen, codegen::Codegen, context::VariableKind, error::LLVMCodegenErrorKind};

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

            ExpressionKind::StringLiteral(reference) => {
                let string_value = codegen.string_intern_pool.resolve_reference_or_err(reference, span)?;

                let string = codegen
                    .llvm_builder
                    .build_global_string_ptr(string_value, "string")
                    .map_err(|err| LLVMCodegenErrorKind::builder_error(err, span))?;

                Ok(string.as_basic_value_enum())
            }

            ExpressionKind::IdentifierReference(reference) => {
                let variable = codegen
                    .context
                    .scope_context(span)?
                    .get_variable(&reference.name, span)?;

                let variable_name = codegen
                    .string_intern_pool
                    .resolve_reference_or_err(&reference.name, span)?;

                // We have the pointer to the variable, we need to dereference that pointer to get the value.
                let value = match variable.kind {
                    VariableKind::Local(pointer) => {
                        if reference.is_reference {
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

            ExpressionKind::BinaryOperation(binary_operation) => binary_operation.codegen(codegen, span),
            ExpressionKind::FunctionCall(call) => call.codegen(codegen, span),
            ExpressionKind::StructureInitialization(initialization) => {
                initialization.codegen(codegen, self.r#type, span)
            }

            #[allow(unreachable_patterns)]
            _ => return LLVMCodegenErrorKind::unable_to_codegen_expression(&self).into(),
        }
    }
}

impl<'ctx> Codegen<'ctx> for BinaryOperation {
    fn codegen(&self, codegen: &mut LLVMCodegen<'ctx>, span: SourceSpan) -> Result<BasicValueEnum<'ctx>> {
        // FIXME: This assumes that both values are integer types.
        let left_value = self.left.codegen(codegen, span)?.into_int_value();
        let right_value = self.right.codegen(codegen, span)?.into_int_value();

        let result = match self.operation {
            Operation::Add => codegen.llvm_builder.build_int_add(left_value, right_value, "add"),
            Operation::Subtract => codegen.llvm_builder.build_int_sub(left_value, right_value, "sub"),
            Operation::Multiply => codegen.llvm_builder.build_int_mul(left_value, right_value, "mul"),
            Operation::Divide => codegen
                .llvm_builder
                .build_int_signed_div(left_value, right_value, "div"),
        }
        .map_err(|err| LLVMCodegenErrorKind::builder_error(err, span))?;

        Ok(result.as_basic_value_enum())
    }
}

trait StructureInitializationCodegen<'ctx> {
    fn codegen(
        &self,
        codegen: &mut LLVMCodegen<'ctx>,
        r#type: Option<TypeReference>,
        span: SourceSpan,
    ) -> Result<BasicValueEnum<'ctx>>;
}

impl<'ctx> StructureInitializationCodegen<'ctx> for StructureInitialization {
    fn codegen(
        &self,
        codegen: &mut LLVMCodegen<'ctx>,
        r#type: Option<TypeReference>,
        span: SourceSpan,
    ) -> Result<BasicValueEnum<'ctx>> {
        let struct_type = codegen.create_value_type(r#type, span)?.into_struct_type();
        Ok(struct_type.const_named_struct(&[]).as_basic_value_enum())
    }
}
