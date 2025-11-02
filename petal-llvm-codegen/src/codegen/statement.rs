use inkwell::values::{BasicValue, BasicValueEnum};
use petal_ast::statement::{
    Statement, StatementKind, function_declaration::FunctionDeclaration, r#return::ReturnStatement,
    variable_declaration::VariableDeclaration,
};
use petal_core::{error::Result, source_span::SourceSpan};

use crate::{
    LLVMCodegen, codegen::Codegen, context::Variable, error::LLVMCodegenErrorKind,
    string_intern_pool_ext::StringInternPoolExt,
};

impl<'ctx> Codegen<'ctx> for Statement {
    fn codegen(&self, codegen: &mut LLVMCodegen<'ctx>, _span: SourceSpan) -> Result<BasicValueEnum<'ctx>> {
        match &self.kind {
            StatementKind::FunctionDeclaration(declaration) => declaration.codegen(codegen, self.span),
            StatementKind::ReturnStatement(return_statement) => return_statement.codegen(codegen, self.span),
            StatementKind::VariableDeclaration(declaration) => declaration.codegen(codegen, self.span),

            #[allow(unreachable_patterns)]
            _ => return LLVMCodegenErrorKind::unable_to_codegen_statement(self).into(),
        }
    }
}

impl<'ctx> Codegen<'ctx> for FunctionDeclaration {
    fn codegen(&self, codegen: &mut LLVMCodegen<'ctx>, span: SourceSpan) -> Result<BasicValueEnum<'ctx>> {
        let function_name = codegen
            .string_intern_pool
            .resolve_reference_or_err(&self.name_reference, span)?;

        let function_type = codegen.create_function_type(self.return_type, &self.parameters)?;
        let function = codegen.llvm_module.add_function(function_name, function_type, None);

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
                .resolve_reference_or_err(&parameter.name_reference, parameter.span)?;

            parameter_value.set_name(parameter_name);

            let pointer = codegen
                .llvm_builder
                .build_alloca(parameter_value.get_type(), parameter_name)
                .map_err(|err| LLVMCodegenErrorKind::builder_error(err, span))?;

            codegen
                .llvm_builder
                .build_store(pointer, *parameter_value)
                .map_err(|err| LLVMCodegenErrorKind::builder_error(err, span))?;

            codegen.context.scope_context(parameter.span)?.declare_variable(
                parameter.name_reference,
                Variable::new(parameter_value.get_type(), pointer),
            );
        }

        for statement in &self.body {
            statement.codegen(codegen, statement.span)?;
        }

        codegen.context.end_scope_context();
        Ok(function.as_global_value().as_pointer_value().into())
    }
}

impl<'ctx> Codegen<'ctx> for ReturnStatement {
    fn codegen(&self, codegen: &mut LLVMCodegen<'ctx>, span: SourceSpan) -> Result<BasicValueEnum<'ctx>> {
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

impl<'ctx> Codegen<'ctx> for VariableDeclaration {
    fn codegen(&self, codegen: &mut LLVMCodegen<'ctx>, span: SourceSpan) -> Result<BasicValueEnum<'ctx>> {
        // We first need to get the value type of the variable, and then allocate some space on the stack for it.
        let value_type = codegen.create_value_type(Some(self.r#type), span)?;

        let variable_name = codegen
            .string_intern_pool
            .resolve_reference_or_err(&self.identifier_reference, span)?;

        let pointer = codegen
            .llvm_builder
            .build_alloca(value_type, variable_name)
            .map_err(|err| LLVMCodegenErrorKind::builder_error(err, span))?;

        // We can then store the initial value into the allocated stack space.
        let initial_value = self.value.codegen(codegen, span)?;

        codegen
            .llvm_builder
            .build_store(pointer, initial_value)
            .map_err(|err| LLVMCodegenErrorKind::builder_error(err, span))?;

        // Finally, now that we've built the variable declaration, we can add the variable to the scope's context.
        codegen
            .context
            .scope_context(span)?
            .declare_variable(self.identifier_reference, Variable::new(value_type, pointer));

        Ok(pointer.as_basic_value_enum())
    }
}
