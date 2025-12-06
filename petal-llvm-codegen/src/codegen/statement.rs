use petal_ast::statement::{r#return::Return, variable_declaration::VariableDeclaration};
use petal_core::{error::Result, source_span::SourceSpan};

use crate::{LLVMCodegen, codegen::StatementCodegen, context::scope::Variable, error::IntoCodegenResult};

impl<'ctx> StatementCodegen<'ctx> for Return {
    fn generate(&self, codegen: &mut LLVMCodegen, span: SourceSpan) -> Result<()> {
        let expression = match self.value.as_ref() {
            Some(value) => value,
            None => {
                // If there is no expression, then we can just emit an empty return.
                codegen.llvm_builder.build_return(None).into_codegen_result(span)?;
                return Ok(());
            }
        };

        let value = codegen.visit_expression(expression)?;

        codegen
            .llvm_builder
            .build_return(Some(&value))
            .into_codegen_result(span)?;

        Ok(())
    }
}

impl<'ctx> StatementCodegen<'ctx> for VariableDeclaration {
    fn generate(&self, codegen: &mut LLVMCodegen, span: SourceSpan) -> Result<()> {
        let variable_name = codegen.string_intern_pool.resolve_reference_or_err(&self.name, span)?;

        // We must be able to convert the variable's type into an LLVM type, and we also must be able to generate its initial value.
        let value_type = codegen.get_basic_llvm_type(&self.r#type)?;
        let value = codegen.visit_expression(&self.value)?;

        // We can then allocate a local variable and store the initial value into it.
        let local = codegen
            .llvm_builder
            .build_alloca(value_type, variable_name)
            .into_codegen_result(span)?;

        codegen
            .llvm_builder
            .build_store(local, value)
            .into_codegen_result(span)?;

        codegen
            .context
            .scope_context(span)?
            .declare_variable(self.name, Variable::local(value_type, local));

        Ok(())
    }
}
