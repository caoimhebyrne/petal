use inkwell::{types::BasicTypeEnum, values::BasicValueEnum};
use petal_ast::node::FunctionCall;
use petal_core::{error::Result, source_span::SourceSpan};

use crate::{
    LLVMCodegen,
    error::{IntoCodegenResult, LLVMCodegenError},
};

pub mod expression;
pub mod statement;
pub mod top_level_statement;

pub trait StatementCodegen<'ctx> {
    /// Generates LLVM IR for a statement node (where the type implementing the trait is the "statement node").
    ///
    /// Parameters:
    /// - codegen: The [LLVMCodegen] instance which is performing the code generation.
    /// - span: The span that the node occurred at within the source code.
    ///
    /// Returns:
    /// - A [Result] indicating whether the operation was successful.
    fn generate(&self, codegen: &mut LLVMCodegen<'ctx>, span: SourceSpan) -> Result<()>;
}

/// Options for generating code for an expression.
#[derive(Debug, Default)]
pub struct ExpressionCodegenOptions {
    /// Whether the generated value should be a reference.
    pub as_reference: bool,
}

impl ExpressionCodegenOptions {
    /// Returns a default set of [ExpressionCodegenOptions], with [ExpressionCodegenOptions::as_reference] set to true.
    pub fn as_reference() -> Self {
        ExpressionCodegenOptions { as_reference: true }
    }
}

pub trait ExpressionCodegen<'ctx> {
    /// Generates LLVM IR for an expression node (where the type implementing the trait is the "expression node").
    ///
    /// Parameters:
    /// - codegen: The [LLVMCodegen] instance which is performing the code generation.
    /// - type: The [BasicTypeEnum] that this node should produce.
    /// - span: The span that the node occurred at within the source code.
    ///
    /// Returns:
    /// - A [Result] wrapping a [BasicValueEnum] that this expression produced.
    fn generate(
        &self,
        codegen: &mut LLVMCodegen<'ctx>,
        r#type: &BasicTypeEnum<'ctx>,
        options: ExpressionCodegenOptions,
        span: SourceSpan,
    ) -> Result<BasicValueEnum<'ctx>>;
}

impl<'ctx> ExpressionCodegen<'ctx> for FunctionCall {
    fn generate(
        &self,
        codegen: &mut LLVMCodegen<'ctx>,
        _type: &BasicTypeEnum<'ctx>,
        _options: ExpressionCodegenOptions,
        span: SourceSpan,
    ) -> Result<BasicValueEnum<'ctx>> {
        let function_name = codegen.string_intern_pool.resolve_reference_or_err(&self.name, span)?;

        let function = codegen
            .llvm_module
            .get_function(function_name)
            .ok_or(LLVMCodegenError::undeclared_function(function_name, span))?;

        let arguments = self
            .arguments
            .iter()
            .map(|it| codegen.visit_expression(it).map(|it| it.into()))
            .collect::<Result<Vec<_>>>()?;

        let function_call = codegen
            .llvm_builder
            .build_call(function, &arguments, function_name)
            .into_codegen_result(span)?;

        // If the value is not a basic value, we can assume that the function returned void.
        Ok(function_call
            .try_as_basic_value()
            .left_or(codegen.llvm_context.bool_type().const_zero().into()))
    }
}

impl<'ctx> StatementCodegen<'ctx> for FunctionCall {
    fn generate(&self, codegen: &mut LLVMCodegen<'ctx>, span: SourceSpan) -> Result<()> {
        ExpressionCodegen::generate(
            self,
            codegen,
            &codegen.llvm_context.bool_type().into(),
            ExpressionCodegenOptions::default(),
            span,
        )?;

        Ok(())
    }
}
