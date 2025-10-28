use inkwell::types::AnyTypeEnum;
use petal_core::error::Result;

use crate::LLVMCodegen;

pub mod statement;

/// Allows a type to implement codegen for itself.
pub trait Codegen<'ctx> {
    /// Generates code using LLVM for a specific type.
    fn codegen(&self, codegen: &'ctx LLVMCodegen) -> Result<AnyTypeEnum<'ctx>>;
}
