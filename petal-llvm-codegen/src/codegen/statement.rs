use petal_ast::statement::{
    StatementNodeKind, r#if::If, r#return::Return, variable_assignment::VariableAssignment,
    variable_declaration::VariableDeclaration, while_loop::WhileLoop,
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

        for statement in &self.then_block {
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

        codegen.llvm_builder.position_at_end(else_block);

        let mut found_return_in_else_block = false;

        for statement in &self.else_block {
            codegen.visit_statement(statement)?;

            if let StatementNodeKind::Return(_) = statement.kind {
                found_return_in_else_block = true;
            }
        }

        if !found_return_in_else_block {
            codegen
                .llvm_builder
                .build_unconditional_branch(end_block)
                .into_codegen_result(span)?;
        }

        if found_return_in_if_block && found_return_in_else_block {
            // Both blocks had a return, we do not need the final end block.
            let _ = end_block.remove_from_function();
        } else {
            // And finally, we can return to the end.
            codegen.llvm_builder.position_at_end(end_block);
        }

        Ok(())
    }
}

impl<'ctx> StatementCodegen<'ctx> for WhileLoop {
    fn generate(&self, codegen: &mut LLVMCodegen<'ctx>, span: SourceSpan) -> Result<()> {
        // Before we do any codegen, we must first set up the blocks that we will need.
        let current_block = codegen.llvm_builder.get_insert_block().expect("");

        let condition_block = codegen.llvm_context.insert_basic_block_after(current_block, "cond");
        let while_block = codegen.llvm_context.insert_basic_block_after(condition_block, "while");
        let end_block = codegen.llvm_context.insert_basic_block_after(while_block, "end");

        // 1. Jump to the condition block.
        codegen
            .llvm_builder
            .build_unconditional_branch(condition_block)
            .into_codegen_result(span)?;

        codegen.llvm_builder.position_at_end(condition_block);

        // 2. Evaluate the condition.
        let condition = codegen.visit_expression(&self.condition)?;

        codegen
            .llvm_builder
            .build_conditional_branch(condition.into_int_value(), while_block, end_block)
            .into_codegen_result(span)?;

        codegen.llvm_builder.position_at_end(while_block);

        for statement in &self.block {
            codegen.visit_statement(statement)?;
        }

        // 3. After the block has been executed, jump back to the condition block.
        codegen
            .llvm_builder
            .build_unconditional_branch(condition_block)
            .into_codegen_result(span)?;

        codegen.llvm_builder.position_at_end(end_block);

        Ok(())
    }
}
