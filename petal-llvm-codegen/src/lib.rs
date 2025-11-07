use std::{env, mem::ManuallyDrop, path::PathBuf};

use inkwell::{
    AddressSpace, OptimizationLevel,
    builder::Builder,
    context::Context,
    module::Module,
    targets::{CodeModel, FileType, InitializationConfig, RelocMode, Target, TargetMachine},
    types::{BasicMetadataTypeEnum, BasicType, BasicTypeEnum, FunctionType},
};
use petal_ast::{
    statement::{Statement, function_declaration::FunctionParameter},
    visitor::ASTVisitor,
};
use petal_codegen_driver::{Driver, options::DriverOptions};
use petal_core::{
    error::Result,
    source_span::SourceSpan,
    string_intern::StringInternPool,
    r#type::{ResolvedType, Type, TypeReference, pool::TypePool},
};

use crate::{codegen::Codegen, context::CodegenContext, error::LLVMCodegenErrorKind};

pub mod codegen;
pub mod context;
pub mod error;

/// An implementation of a code generator which produces a final binary using LLVM.
pub struct LLVMCodegen<'ctx> {
    /// The [DriverOptions] passed to this LLVM codegen driver.
    pub(crate) driver_options: DriverOptions,

    /// The [TypePool] to read types from.
    pub(crate) type_pool: &'ctx TypePool,

    /// The StringInternPool implementation to use when reading identifiers.
    pub(crate) string_intern_pool: &'ctx dyn StringInternPool,

    /// The [CodegenContext] used during the compilation.
    pub(crate) context: CodegenContext<'ctx>,

    /// The [LLVMContextHolder] which contains the LLVM [Context] to be used by this codegen.
    pub(crate) llvm_context: &'ctx Context,

    /// The LLVM module being used.
    pub(crate) llvm_module: ManuallyDrop<Module<'ctx>>,

    /// The builder being used.
    pub(crate) llvm_builder: ManuallyDrop<Builder<'ctx>>,
}

impl<'ctx> LLVMCodegen<'ctx> {
    /// Converts the provided type to a function type.
    pub fn create_function_type(
        &self,
        type_reference: &TypeReference,
        parameters: &Vec<FunctionParameter>,
    ) -> Result<FunctionType<'ctx>> {
        let type_kind = self.ensure_resolved(Some(*type_reference), type_reference.span)?;

        let mut parameter_types: Vec<BasicMetadataTypeEnum<'ctx>> = Vec::new();

        for parameter in parameters {
            let parameter_type = self
                .create_value_type(Some(parameter.value_type), parameter.span)?
                .into();

            parameter_types.push(parameter_type);
        }

        let llvm_type = match type_kind {
            ResolvedType::Integer(size) => self
                .llvm_context
                .custom_width_int_type(size)
                .fn_type(&parameter_types, false),

            ResolvedType::Void => self.llvm_context.void_type().fn_type(&parameter_types, false),

            ResolvedType::Reference(_) => self
                .llvm_context
                .ptr_type(AddressSpace::default())
                .fn_type(&parameter_types, false),
        };

        Ok(llvm_type)
    }

    /// Converts the provided type to a value type.
    pub fn create_value_type(
        &self,
        maybe_type_reference: Option<TypeReference>,
        span: SourceSpan,
    ) -> Result<BasicTypeEnum<'ctx>> {
        let type_kind = self.ensure_resolved(maybe_type_reference, span)?;

        let llvm_type = match type_kind {
            ResolvedType::Integer(size) => self.llvm_context.custom_width_int_type(size).as_basic_type_enum(),

            ResolvedType::Reference(_) => self.llvm_context.ptr_type(AddressSpace::default()).as_basic_type_enum(),

            _ => return LLVMCodegenErrorKind::bad_value_type(type_kind, span).into(),
        };

        Ok(llvm_type)
    }

    /// Asserts that the provided type is resolved, returning an error if it is not.
    pub fn ensure_resolved(
        &self,
        maybe_type_reference: Option<TypeReference>,
        span: SourceSpan,
    ) -> Result<ResolvedType> {
        let reference = maybe_type_reference.ok_or(LLVMCodegenErrorKind::unresolved_type("missing", span))?;
        let r#type = self.type_pool.get_type_or_err(&reference.id, span)?;

        let kind = match r#type {
            Type::Resolved(kind) => kind,

            Type::Unresolved(reference) => {
                let type_name = self.string_intern_pool.resolve_reference_or_err(&reference, span)?;
                return LLVMCodegenErrorKind::unresolved_type(type_name, span).into();
            }
        };

        Ok(*kind)
    }
}

impl<'ctx> Driver<'ctx> for LLVMCodegen<'ctx> {
    fn new(options: DriverOptions, type_pool: &'ctx TypePool, string_intern_pool: &'ctx dyn StringInternPool) -> Self {
        // We're creating and leaking the LLVM context here. The `Drop` implementation of this struct will ensure that
        // this is cleaned up successfully.
        let llvm_context: &'ctx Context = Box::leak(Box::new(Context::create()));

        LLVMCodegen {
            llvm_context,
            llvm_module: ManuallyDrop::new(llvm_context.create_module(&options.module_name)),
            llvm_builder: ManuallyDrop::new(llvm_context.create_builder()),
            type_pool,
            string_intern_pool,
            driver_options: options,
            context: CodegenContext::new(),
        }
    }

    fn compile_to_object(&self) -> std::result::Result<PathBuf, String> {
        if let Err(error) = self.llvm_module.verify() {
            return Err(error.to_string());
        }

        if self.driver_options.dump_bytecode {
            println!("{}", self.llvm_module.print_to_string().to_string());
        }

        // We will write the object file to a temporary path, the caller is responsible for linking the object file into
        // an executable.
        let mut object_file_path = env::temp_dir();
        object_file_path.push(format!("petal-{}.o", self.driver_options.module_name));

        // We can then compile the LLVM module into that file path.
        Target::initialize_all(&InitializationConfig::default());

        let triple = TargetMachine::get_default_triple();
        let target = Target::from_triple(&triple).map_err(|it| it.to_string())?;
        let target_machine = target
            .create_target_machine(
                &triple,
                "",
                "",
                OptimizationLevel::None,
                RelocMode::Default,
                CodeModel::Default,
            )
            .ok_or("Failed to create LLVM target machine".to_string())?;

        target_machine
            .write_to_file(&self.llvm_module, FileType::Object, &object_file_path)
            .map_err(|it| it.to_string())?;

        Ok(object_file_path)
    }
}

impl<'ctx> ASTVisitor for LLVMCodegen<'ctx> {
    fn visit(&mut self, statement: &mut Statement) -> Result<()> {
        statement.codegen(self, statement.span).map(|_| ())
    }
}

/// The drop implementation ensures that the context that was leaked in the constructor is cleaned up properly.
impl<'ctx> Drop for LLVMCodegen<'ctx> {
    fn drop(&mut self) {
        unsafe {
            // SAFETY: These values are not dropped anywhere other than in this method. This means that these values are
            // valid during all execution up until the struct holding them is dropped.
            ManuallyDrop::drop(&mut self.llvm_builder);
            ManuallyDrop::drop(&mut self.llvm_module);

            // SAFETY: We know that the `llvm_context` within the codegen was created using [Box::leak] in the `new`
            // factory function. We have also already dropped the builder and module manually.
            let context_box = Box::from_raw(self.llvm_context as *const Context as *mut Context);
            drop(context_box);
        }
    }
}
