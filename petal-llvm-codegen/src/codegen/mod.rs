use inkwell::values::BasicValueEnum;
use petal_core::{error::Result, source_span::SourceSpan};

use crate::LLVMCodegen;

pub mod expression;
pub mod statement;

/// Allows a type to implement codegen for itself.
pub trait Codegen<'ctx> {
    /// Generates code using LLVM for a specific type.
    fn codegen(&self, codegen: &mut LLVMCodegen<'ctx>, span: SourceSpan) -> Result<BasicValueEnum<'ctx>>;
}
