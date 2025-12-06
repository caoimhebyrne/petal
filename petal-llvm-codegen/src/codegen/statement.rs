use petal_ast::statement::r#return::Return;
use petal_core::{error::Result, source_span::SourceSpan};

use crate::{LLVMCodegen, codegen::StatementCodegen};

impl<'ctx> StatementCodegen<'ctx> for Return {
    fn generate(&self, codegen: &mut LLVMCodegen, _span: SourceSpan) -> Result<()> {
        let expression = match self.value.as_ref() {
            Some(value) => value,
            None => {
                // If there is no expression, then we can just emit an empty return.
                codegen.llvm_builder.build_return(None);
                return Ok(());
            }
        };

        let value = codegen.visit_expression(expression)?;
        codegen.llvm_builder.build_return(Some(&value));

        Ok(())
    }
}
