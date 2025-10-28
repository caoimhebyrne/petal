use inkwell::{
    builder::Builder,
    context::Context,
    module::Module,
    types::{BasicType, BasicTypeEnum, FunctionType},
};
use petal_ast::{
    statement::Statement,
    r#type::{ResolvedTypeKind, Type, TypeKind},
    visitor::ASTVisitor,
};
use petal_core::{error::Result, source_span::SourceSpan, string_intern::StringInternPool};

use crate::{codegen::Codegen, error::LLVMCodegenErrorKind, string_intern_pool_ext::StringInternPoolExt};

pub mod codegen;
pub mod error;
pub mod string_intern_pool_ext;

/// The context passed to an [LLVMCodegen] during initialization.
pub struct LLVMCodegenContext {
    /// The LLVM context that contains all of the entities within the LLVM API (like the module).
    /// This MUST outlive the [LLVMCodegen] struct that contains the module and builder, hence why it's in here.
    pub(crate) llvm_context: Context,

    /// Whether the module's bytecode should be dumped to stderr before compilation.
    pub(crate) dump_bytecode: bool,
}

impl LLVMCodegenContext {
    /// Creates a new [LLVMCodegenContext].
    pub fn new(dump_bytecode: bool) -> Self {
        LLVMCodegenContext {
            llvm_context: Context::create(),
            dump_bytecode,
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

        if self.codegen_context.dump_bytecode {
            println!("{}", self.llvm_module.print_to_string().to_string());
        }

        // TODO: Create the target machine, write the object file to a temporary path and return that path.
        Ok(())
    }

    /// Converts the provided type to a function type.
    /// TODO: Include parameters.
    pub fn create_function_type(&self, r#type: Type) -> Result<FunctionType<'ctx>> {
        let (type_kind, _) = self.ensure_resolved(Some(r#type), r#type.span)?;

        let llvm_type = match type_kind {
            ResolvedTypeKind::Integer(size) => self
                .codegen_context
                .llvm_context
                .custom_width_int_type(size)
                .fn_type(&[], false),

            ResolvedTypeKind::Void => self.codegen_context.llvm_context.void_type().fn_type(&[], false),
        };

        Ok(llvm_type)
    }

    /// Converts the provided type to a value type.
    pub fn create_value_type(&self, maybe_type: Option<Type>, span: SourceSpan) -> Result<BasicTypeEnum<'ctx>> {
        let (type_kind, type_span) = self.ensure_resolved(maybe_type, span)?;

        let llvm_type = match type_kind {
            ResolvedTypeKind::Integer(size) => self
                .codegen_context
                .llvm_context
                .custom_width_int_type(size)
                .as_basic_type_enum(),

            _ => return LLVMCodegenErrorKind::bad_value_type(type_kind, type_span).into(),
        };

        Ok(llvm_type)
    }

    /// Asserts that the provided type is resolved, returning an error if it is not.
    pub fn ensure_resolved(
        &self,
        maybe_type: Option<Type>,
        span: SourceSpan,
    ) -> Result<(ResolvedTypeKind, SourceSpan)> {
        let r#type = maybe_type.ok_or(LLVMCodegenErrorKind::unresolved_type("missing", span))?;

        let kind = match r#type.kind {
            TypeKind::Resolved(kind) => kind,
            TypeKind::Unresolved(reference) => {
                let type_name = self
                    .string_intern_pool
                    .resolve_reference_or_err(&reference, r#type.span)?;

                return LLVMCodegenErrorKind::unresolved_type(type_name, r#type.span).into();
            }
        };

        Ok((kind, r#type.span))
    }
}

impl<'ctx> ASTVisitor for LLVMCodegen<'ctx> {
    fn visit(&mut self, statement: &mut Statement) -> Result<()> {
        statement.codegen(self, statement.span).map(|_| ())
    }
}
