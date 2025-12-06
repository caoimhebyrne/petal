use petal_ast::statement::function_declaration::FunctionDeclaration;
use petal_core::{error::Result, source_span::SourceSpan, r#type::TypeReference};

use crate::{LLVMCodegen, codegen::StatementCodegen};

impl<'ctx> StatementCodegen<'ctx> for FunctionDeclaration {
    fn generate(&self, codegen: &mut LLVMCodegen, span: SourceSpan) -> Result<()> {
        // We must be able to create a function type from the statement.
        let parameter_types: Vec<TypeReference> = self.parameters.iter().map(|it| it.r#type).collect();

        let function_type = codegen.create_function_type(&self.return_type, &parameter_types)?;
        let function_name = codegen.string_intern_pool.resolve_reference_or_err(&self.name, span)?;

        // We can then create a function and start constructing its body (if it is not external).
        let function = codegen.llvm_module.add_function(function_name, function_type, None);

        if !self.is_external() {
            let entry_block = codegen.llvm_context.append_basic_block(function, "entry");
            codegen.llvm_builder.position_at_end(entry_block);

            codegen.context.start_scope_context();

            for statement in &self.body {
                codegen.visit_statement(statement)?;
            }

            codegen.context.end_scope_context();
        }

        Ok(())
    }
}
