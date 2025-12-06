use inkwell::{module::Linkage, values::BasicValueEnum};
use petal_ast::statement::{
    TopLevelStatementNode, TopLevelStatementNodeKind, function_declaration::FunctionDeclaration,
};
use petal_core::{error::Result, source_span::SourceSpan};

use crate::{LLVMCodegen, codegen::Codegen, context::Variable};

impl<'ctx> Codegen<'ctx> for TopLevelStatementNode {
    fn codegen(
        &self,
        codegen: &mut LLVMCodegen<'ctx>,
        _span: SourceSpan,
        _as_reference: bool,
    ) -> Result<BasicValueEnum<'ctx>> {
        match &self.kind {
            TopLevelStatementNodeKind::FunctionDeclaration(function) => function.codegen(codegen, self.span, false),
            TopLevelStatementNodeKind::Import(_) => Ok(codegen.llvm_context.bool_type().const_zero().into()),
        }
    }
}

impl<'ctx> Codegen<'ctx> for FunctionDeclaration {
    fn codegen(
        &self,
        codegen: &mut LLVMCodegen<'ctx>,
        span: SourceSpan,
        _as_reference: bool,
    ) -> Result<BasicValueEnum<'ctx>> {
        let function_name = codegen.string_intern_pool.resolve_reference_or_err(&self.name, span)?;
        let function_type = codegen.create_function_type(&self.return_type, &self.parameters)?;

        let linkage = if self.is_external() {
            Some(Linkage::External)
        } else {
            None
        };

        let function = codegen.llvm_module.add_function(function_name, function_type, linkage);

        if !self.is_external() {
            let block = codegen.llvm_context.append_basic_block(function, "entry");
            codegen.llvm_builder.position_at_end(block);
            codegen.context.start_scope_context();

            for (index, parameter_value) in function.get_params().iter().enumerate() {
                let parameter = self
                    .parameters
                    .iter()
                    .nth(index)
                    .expect("LLVM parameters did not match function params!");

                let parameter_name = codegen
                    .string_intern_pool
                    .resolve_reference_or_err(&parameter.name, parameter.span)?;

                parameter_value.set_name(parameter_name);

                codegen.context.scope_context(parameter.span)?.declare_variable(
                    parameter.name,
                    Variable::parameter(parameter_value.get_type(), *parameter_value),
                );
            }

            for statement in &self.body {
                statement.codegen(codegen, statement.span, false)?;
            }

            codegen.context.end_scope_context();
        }

        Ok(function.as_global_value().as_pointer_value().into())
    }
}
