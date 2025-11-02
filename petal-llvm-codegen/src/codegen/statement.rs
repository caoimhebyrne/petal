use inkwell::values::BasicValueEnum;
use petal_ast::statement::{
    Statement, StatementKind, function_declaration::FunctionDeclaration, r#return::ReturnStatement,
};
use petal_core::{error::Result, source_span::SourceSpan};

use crate::{LLVMCodegen, codegen::Codegen, error::LLVMCodegenErrorKind, string_intern_pool_ext::StringInternPoolExt};

impl<'ctx> Codegen<'ctx> for Statement {
    fn codegen(&self, codegen: &'ctx LLVMCodegen, _span: SourceSpan) -> Result<BasicValueEnum<'ctx>> {
        match &self.kind {
            StatementKind::FunctionDeclaration(declaration) => declaration.codegen(codegen, self.span),
            StatementKind::ReturnStatement(return_statement) => return_statement.codegen(codegen, self.span),

            _ => return LLVMCodegenErrorKind::unable_to_codegen_statement(self).into(),
        }
    }
}

impl<'ctx> Codegen<'ctx> for FunctionDeclaration {
    fn codegen(&self, codegen: &'ctx LLVMCodegen, span: SourceSpan) -> Result<BasicValueEnum<'ctx>> {
        let function_name = codegen
            .string_intern_pool
            .resolve_reference_or_err(&self.name_reference, span)?;

        let function_type = codegen.create_function_type(self.return_type)?;
        let function = codegen.llvm_module.add_function(function_name, function_type, None);

        let block = codegen.llvm_context.append_basic_block(function, "entry");
        codegen.llvm_builder.position_at_end(block);

        for statement in &self.body {
            statement.codegen(codegen, statement.span)?;
        }

        Ok(function.as_global_value().as_pointer_value().into())
    }
}

impl<'ctx> Codegen<'ctx> for ReturnStatement {
    fn codegen(&self, codegen: &'ctx LLVMCodegen, span: SourceSpan) -> Result<BasicValueEnum<'ctx>> {
        if let Some(return_value) = self.value.as_ref().map(|it| it.codegen(codegen, it.span)).transpose()? {
            codegen.llvm_builder.build_return(Some(&return_value))
        } else {
            codegen.llvm_builder.build_return(None)
        }
        .map_err(|err| LLVMCodegenErrorKind::builder_error(err, span))?;

        // This is the 'unit' type. I would prefer not to have to do this, but we need to return a BasicValueEnum.
        Ok(codegen.llvm_context.bool_type().const_zero().into())
    }
}
