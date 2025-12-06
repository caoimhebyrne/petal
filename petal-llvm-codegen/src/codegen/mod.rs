use inkwell::{types::BasicTypeEnum, values::BasicValueEnum};
use petal_core::{error::Result, source_span::SourceSpan};

use crate::LLVMCodegen;

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
    fn generate(&self, codegen: &mut LLVMCodegen, span: SourceSpan) -> Result<()>;
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
        codegen: &mut LLVMCodegen,
        r#type: &BasicTypeEnum<'ctx>,
        span: SourceSpan,
    ) -> Result<BasicValueEnum<'ctx>>;
}
