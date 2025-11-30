use inkwell::values::{BasicMetadataValueEnum, BasicValueEnum};
use petal_ast::node::FunctionCall;
use petal_core::{error::Result, source_span::SourceSpan};

use crate::{LLVMCodegen, error::LLVMCodegenErrorKind};

pub mod expression;
pub mod statement;
pub mod top_level_statement;

/// Allows a type to implement codegen for itself.
pub trait Codegen<'ctx> {
    /// Generates code using LLVM for a specific type.
    fn codegen(
        &self,
        codegen: &mut LLVMCodegen<'ctx>,
        span: SourceSpan,
        as_reference: bool,
    ) -> Result<BasicValueEnum<'ctx>>;
}

/// A function call is both a statement and an expression, so it does not belong in either submodule.
impl<'ctx> Codegen<'ctx> for FunctionCall {
    fn codegen(
        &self,
        codegen: &mut LLVMCodegen<'ctx>,
        span: SourceSpan,
        _as_reference: bool,
    ) -> Result<BasicValueEnum<'ctx>> {
        let function_name = codegen.string_intern_pool.resolve_reference_or_err(&self.name, span)?;

        let function = codegen
            .llvm_module
            .get_function(function_name)
            .ok_or(LLVMCodegenErrorKind::undeclared_function(function_name, span))?;

        let mut arguments: Vec<BasicMetadataValueEnum<'_>> = vec![];
        for argument in &self.arguments {
            arguments.push(argument.codegen(codegen, argument.span, false)?.into());
        }

        let function_call = codegen
            .llvm_builder
            .build_call(function, &arguments, function_name)
            .map_err(|err| LLVMCodegenErrorKind::builder_error(err, span))?;

        // If we cannot get a basic value from the function call, we can assume it is a void function and therefore
        // should use the unit type.
        Ok(function_call
            .try_as_basic_value()
            .left_or(codegen.llvm_context.bool_type().const_zero().into()))
    }
}
