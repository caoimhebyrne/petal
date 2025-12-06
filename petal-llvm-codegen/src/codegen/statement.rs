use inkwell::values::{BasicValue, BasicValueEnum};
use petal_ast::statement::{
    StatementNode, StatementNodeKind, r#return::Return, variable_assignment::VariableAssignment,
    variable_declaration::VariableDeclaration,
};
use petal_core::{error::Result, source_span::SourceSpan};

use crate::{
    LLVMCodegen,
    codegen::Codegen,
    context::{Variable, VariableKind},
    error::LLVMCodegenErrorKind,
};

impl<'ctx> Codegen<'ctx> for StatementNode {
    fn codegen(
        &self,
        codegen: &mut LLVMCodegen<'ctx>,
        _span: SourceSpan,
        _as_reference: bool,
    ) -> Result<BasicValueEnum<'ctx>> {
        match &self.kind {
            StatementNodeKind::Return(r#return) => r#return.codegen(codegen, self.span, false),
            StatementNodeKind::VariableDeclaration(declaration) => declaration.codegen(codegen, self.span, false),
            StatementNodeKind::FunctionCall(call) => call.codegen(codegen, self.span, false),
            StatementNodeKind::VariableAssignment(assignment) => assignment.codegen(codegen, self.span, false),

            #[allow(unreachable_patterns)]
            _ => return LLVMCodegenErrorKind::unable_to_codegen_statement(self).into(),
        }
    }
}

impl<'ctx> Codegen<'ctx> for Return {
    fn codegen(
        &self,
        codegen: &mut LLVMCodegen<'ctx>,
        span: SourceSpan,
        _as_reference: bool,
    ) -> Result<BasicValueEnum<'ctx>> {
        if let Some(return_value) = self
            .value
            .as_ref()
            .map(|it| it.codegen(codegen, it.span, false))
            .transpose()?
        {
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
    fn codegen(
        &self,
        codegen: &mut LLVMCodegen<'ctx>,
        span: SourceSpan,
        _as_reference: bool,
    ) -> Result<BasicValueEnum<'ctx>> {
        // We first need to get the value type of the variable, and then allocate some space on the stack for it.
        let value_type = codegen.resolve_and_create_value_type(Some(self.r#type), span)?;
        let variable_name = codegen.string_intern_pool.resolve_reference_or_err(&self.name, span)?;

        let pointer = codegen
            .llvm_builder
            .build_alloca(value_type, variable_name)
            .map_err(|err| LLVMCodegenErrorKind::builder_error(err, span))?;

        // We can then store the initial value into the allocated stack space.
        let initial_value = self.value.codegen(codegen, span, false)?;

        codegen
            .llvm_builder
            .build_store(pointer, initial_value)
            .map_err(|err| LLVMCodegenErrorKind::builder_error(err, span))?;

        // Finally, now that we've built the variable declaration, we can add the variable to the scope's context.
        codegen
            .context
            .scope_context(span)?
            .declare_variable(self.name, Variable::local(value_type, pointer));

        Ok(pointer.as_basic_value_enum())
    }
}

impl<'ctx> Codegen<'ctx> for VariableAssignment {
    fn codegen(
        &self,
        codegen: &mut LLVMCodegen<'ctx>,
        span: SourceSpan,
        _as_reference: bool,
    ) -> Result<BasicValueEnum<'ctx>> {
        let variable = codegen.context.scope_context(span)?.get_variable(&self.name, span)?;

        let pointer = match variable.kind {
            VariableKind::Local(pointer) => pointer,

            VariableKind::Parameter(parameter) if variable.value_type.is_pointer_type() => {
                parameter.into_pointer_value()
            }

            _ => return LLVMCodegenErrorKind::illegal_variable_assignment(span).into(),
        };

        let value = self.value.codegen(codegen, self.value.span, false)?;

        codegen
            .llvm_builder
            .build_store(pointer, value)
            .map_err(|err| LLVMCodegenErrorKind::builder_error(err, span))?;

        Ok(pointer.as_basic_value_enum())
    }
}
