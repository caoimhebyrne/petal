use inkwell::{builder::Builder, context::Context, module::Module};
use petal_ast::{statement::Statement, visitor::ASTVisitor};
use petal_core::{error::Result, string_intern::StringInternPool};

use crate::codegen::Codegen;

pub mod codegen;

/// The context passed to an [LLVMCodegen] during initialization.
pub struct LLVMCodegenContext {
    /// The LLVM context that contains all of the entities within the LLVM API (like the module).
    /// This MUST outlive the [LLVMCodegen] struct that contains the module and builder, hence why it's in here.
    pub(crate) llvm_context: Context,
}

impl LLVMCodegenContext {
    /// Creates a new [LLVMCodegenContext].
    pub fn new() -> Self {
        LLVMCodegenContext {
            llvm_context: Context::create(),
        }
    }
}

/// An implementation of a code generator which produces a final binary using LLVM.
pub struct LLVMCodegen<'ctx> {
    /// The context for this codegen, this includes the LLVM context used to create the module and builder.
    pub(crate) codegen_context: &'ctx LLVMCodegenContext,

    /// The StringInternPool implementation to use when reading identifiers.
    pub(crate) string_intern_pool: &'ctx dyn StringInternPool,

    /// The LLVM module being used.
    pub(crate) llvm_module: Module<'ctx>,

    /// The builder being used.
    pub(crate) llvm_builder: Builder<'ctx>,
}

impl<'ctx> LLVMCodegen<'ctx> {
    /// Creates a new [LLVMCodegen] instance given a [LLVMCodegenContext].
    pub fn new(codegen_context: &'ctx LLVMCodegenContext, string_intern_pool: &'ctx dyn StringInternPool) -> Self {
        LLVMCodegen {
            codegen_context,
            string_intern_pool,
            // TODO: Derive the module name from the input file's name.
            llvm_module: codegen_context.llvm_context.create_module("module"),
            llvm_builder: codegen_context.llvm_context.create_builder(),
        }
    }

    /// Attempts to compile the generated LLVM module into an object file that can be linked.
    /// This returns a plain [core::result::Result] as no source code information can be provided at this stage.
    pub fn compile(&self) -> core::result::Result<(), String> {
        if let Err(error) = self.llvm_module.verify() {
            return Err(error.to_string());
        }

        // TODO: Create the target machine, write the object file to a temporary path and return that path.
        Ok(())
    }
}

impl<'ctx> ASTVisitor for LLVMCodegen<'ctx> {
    fn visit(&mut self, statement: &mut Statement) -> Result<()> {
        statement.codegen(self).map(|_| ())
    }
}
