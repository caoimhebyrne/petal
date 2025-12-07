use petal_ast::statement::{
    StatementNodeKind, r#if::If, r#return::Return, variable_assignment::VariableAssignment,
    variable_declaration::VariableDeclaration,
};
use petal_core::{error::Result, source_span::SourceSpan};

use crate::{
    LLVMCodegen,
    codegen::StatementCodegen,
    context::scope::{Variable, VariableKind},
    error::{IntoCodegenResult, LLVMCodegenError},
};

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

impl<'ctx> StatementCodegen<'ctx> for VariableAssignment {
    fn generate(&self, codegen: &mut LLVMCodegen<'ctx>, span: SourceSpan) -> Result<()> {
        let variable = codegen.context.scope_context(span)?.get_variable(&self.name, span)?;

        // The variable must not be a parameter.
        let pointer = match variable.kind {
            VariableKind::Local(pointer) => pointer,
            VariableKind::Parameter(parameter) if parameter.is_pointer_value() => parameter.into_pointer_value(),
            _ => return LLVMCodegenError::unable_to_assign_to_parameter(span).into(),
        };

        let value = codegen.visit_expression(&self.value)?;

        codegen
            .llvm_builder
            .build_store(pointer, value)
            .into_codegen_result(span)?;

        Ok(())
    }
}

impl<'ctx> StatementCodegen<'ctx> for If {
    fn generate(&self, codegen: &mut LLVMCodegen<'ctx>, span: SourceSpan) -> Result<()> {
        // Before we do any codegen, we must first set up the blocks that we will need.
        let current_block = codegen.llvm_builder.get_insert_block().expect("");

        let then_block = codegen.llvm_context.insert_basic_block_after(current_block, "then");
        let else_block = codegen.llvm_context.insert_basic_block_after(then_block, "else");
        let end_block = codegen.llvm_context.insert_basic_block_after(else_block, "end");

        let condition = codegen.visit_expression(&self.condition)?;

        codegen
            .llvm_builder
            .build_conditional_branch(condition.into_int_value(), then_block, else_block)
            .into_codegen_result(span)?;

        // Then, we can generate the true block.
        codegen.llvm_builder.position_at_end(then_block);

        let mut found_return_in_if_block = false;

        for statement in &self.block {
            codegen.visit_statement(statement)?;

            if let StatementNodeKind::Return(_) = statement.kind {
                found_return_in_if_block = true;
            }
        }

        if !found_return_in_if_block {
            codegen
                .llvm_builder
                .build_unconditional_branch(end_block)
                .into_codegen_result(span)?;
        }

        // TODO: After we generate the true block, we can generate the else block.

        codegen.llvm_builder.position_at_end(else_block);

        // let mut found_return_in_if_block = false;

        // ...

        // if !found_return_in_else_block {

        codegen
            .llvm_builder
            .build_unconditional_branch(end_block)
            .into_codegen_result(span)?;

        // }

        // And finally, we can return to the end.
        codegen.llvm_builder.position_at_end(end_block);

        Ok(())
    }
}
