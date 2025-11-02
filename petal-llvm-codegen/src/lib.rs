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
use petal_codegen_driver::{Driver, options::DriverOptions};
use petal_core::{error::Result, source_span::SourceSpan, string_intern::StringInternPool};

use crate::{codegen::Codegen, error::LLVMCodegenErrorKind, string_intern_pool_ext::StringInternPoolExt};

pub mod codegen;
pub mod error;
pub mod string_intern_pool_ext;

/// An implementation of a code generator which produces a final binary using LLVM.
pub struct LLVMCodegen<'s, 'ctx> {
    /// The [DriverOptions] passed to this LLVM codegen driver.
    pub(crate) driver_options: DriverOptions,

    /// The StringInternPool implementation to use when reading identifiers.
    pub(crate) string_intern_pool: &'s dyn StringInternPool,

    /// The [LLVMContextHolder] which contains the LLVM [Context] to be used by this codegen.
    pub(crate) llvm_context: &'ctx Context,

    /// The LLVM module being used.
    pub(crate) llvm_module: Module<'ctx>,

    /// The builder being used.
    pub(crate) llvm_builder: Builder<'ctx>,
}

impl<'s, 'ctx> LLVMCodegen<'s, 'ctx> {
    /// Converts the provided type to a function type.
    /// TODO: Include parameters.
    pub fn create_function_type(&self, r#type: Type) -> Result<FunctionType<'ctx>> {
        let (type_kind, _) = self.ensure_resolved(Some(r#type), r#type.span)?;

        let llvm_type = match type_kind {
            ResolvedTypeKind::Integer(size) => self.llvm_context.custom_width_int_type(size).fn_type(&[], false),
            ResolvedTypeKind::Void => self.llvm_context.void_type().fn_type(&[], false),
        };

        Ok(llvm_type)
    }

    /// Converts the provided type to a value type.
    pub fn create_value_type(&self, maybe_type: Option<Type>, span: SourceSpan) -> Result<BasicTypeEnum<'ctx>> {
        let (type_kind, type_span) = self.ensure_resolved(maybe_type, span)?;

        let llvm_type = match type_kind {
            ResolvedTypeKind::Integer(size) => self.llvm_context.custom_width_int_type(size).as_basic_type_enum(),

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

impl<'s, 'ctx> Driver<'s> for LLVMCodegen<'s, 'ctx> {
    fn new(options: DriverOptions, string_intern_pool: &'s dyn StringInternPool) -> Self {
        // We're creating and leaking the LLVM context here. The `Drop` implementation of this struct will ensure that
        // this is cleaned up successfully.
        let llvm_context: &'ctx Context = Box::leak(Box::new(Context::create()));

        LLVMCodegen {
            string_intern_pool,
            llvm_context,
            llvm_module: llvm_context.create_module(&options.module_name),
            llvm_builder: llvm_context.create_builder(),
            driver_options: options,
        }
    }

    fn compile_to_object(&self) -> std::result::Result<std::path::PathBuf, String> {
        if let Err(error) = self.llvm_module.verify() {
            return Err(error.to_string());
        }

        if self.driver_options.dump_bytecode {
            println!("{}", self.llvm_module.print_to_string().to_string());
        }

        // TODO: Create the target machine, write the object file to a temporary path and return that path.
        Err("Compiling to an object file has not been implemented yet.".to_owned())
    }
}

impl<'s, 'ctx> ASTVisitor for LLVMCodegen<'s, 'ctx> {
    fn visit(&mut self, statement: &mut Statement) -> Result<()> {
        statement.codegen(self, statement.span).map(|_| ())
    }
}

/// The drop implementation ensures that the context that was leaked in the constructor is cleaned up properly.
impl<'s, 'ctx> Drop for LLVMCodegen<'s, 'ctx> {
    fn drop(&mut self) {
        // SAFETY: We know that the `llvm_context` within the codegen was created using [Box::leak] in the `new` factory
        // function.
        let context_box = unsafe { Box::from_raw(self.llvm_context as *const Context as *mut Context) };
        drop(context_box);
    }
}
