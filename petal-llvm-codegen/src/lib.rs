use inkwell::{builder::Builder, context::Context, module::Module};
use petal_ast::{statement::Statement, visitor::ASTVisitor};
use petal_core::error::Result;

/// The context passed to an [LLVMCodegen] during initialization.
pub struct LLVMCodegenContext {
    /// The LLVM context that contains all of the entities within the LLVM API (like the module).
    /// This MUST outlive the [LLVMCodegen] struct that contains the module and builder, hence why it's in here.
    pub(crate) context: Context,
}

impl LLVMCodegenContext {
    /// Creates a new [LLVMCodegenContext].
    pub fn new() -> Self {
        LLVMCodegenContext {
            context: Context::create(),
        }
    }
}

/// An implementation of a code generator which produces a final binary using LLVM.
pub struct LLVMCodegen<'ctx> {
    /// The context for this codegen, this includes the LLVM context used to create the module and builder.
    pub(crate) codegen_context: &'ctx LLVMCodegenContext,

    /// The LLVM module being used.
    pub(crate) module: Module<'ctx>,

    /// The builder being used.
    pub(crate) builder: Builder<'ctx>,
}

impl<'ctx> LLVMCodegen<'ctx> {
    /// Creates a new [LLVMCodegen] instance given a [LLVMCodegenContext].
    pub fn new(codegen_context: &'ctx LLVMCodegenContext) -> Self {
        LLVMCodegen {
            codegen_context,
            // TODO: Derive the module name from the input file's name.
            module: codegen_context.context.create_module("module"),
            builder: codegen_context.context.create_builder(),
        }
    }
}

impl<'ctx> ASTVisitor for LLVMCodegen<'ctx> {
    fn visit(&mut self, statement: &mut Statement) -> Result<()> {
        todo!()
    }
}
