use inkwell::types::{AnyType, AnyTypeEnum};
use petal_ast::statement::{Statement, StatementKind, function_declaration::FunctionDeclaration};
use petal_core::error::Result;

use crate::{LLVMCodegen, codegen::Codegen};

impl<'ctx> Codegen<'ctx> for Statement {
    fn codegen(&self, codegen: &'ctx LLVMCodegen) -> Result<AnyTypeEnum<'ctx>> {
        match &self.kind {
            StatementKind::FunctionDeclaration(declaration) => declaration.codegen(codegen),

            // TODO: Return an error indicatng that the statement is not currently supported by the codegen module.
            _ => todo!(),
        }
    }
}

impl<'ctx> Codegen<'ctx> for FunctionDeclaration {
    fn codegen(&self, codegen: &'ctx LLVMCodegen) -> Result<AnyTypeEnum<'ctx>> {
        // TODO: Add an extension to `StringInternPool` which returns an error.
        let function_name = codegen
            .string_intern_pool
            .resolve_reference(&self.name_reference)
            .unwrap();

        // TODO: Use the function's actual return type.
        let return_type = codegen.codegen_context.llvm_context.void_type();
        let function_type = return_type.fn_type(&[], false);

        let function = codegen.llvm_module.add_function(function_name, function_type, None);

        codegen
            .codegen_context
            .llvm_context
            .append_basic_block(function, "entry");

        for statement in &self.body {
            statement.codegen(codegen)?;
        }

        Ok(function_type.as_any_type_enum())
    }
}
