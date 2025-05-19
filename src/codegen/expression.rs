use super::{Codegen, error::CodegenError, r#type::TypeCodegen};
use crate::ast::node::kind::{IdentifierReferenceNode, IntegerLiteralNode};
use inkwell::values::{BasicValue, BasicValueEnum};

pub trait ExpressionCodegen {
    fn codegen<'ctx>(&self, codegen: &mut Codegen<'ctx>) -> Result<BasicValueEnum<'ctx>, CodegenError>;
}

impl ExpressionCodegen for IntegerLiteralNode {
    fn codegen<'ctx>(&self, codegen: &mut Codegen<'ctx>) -> Result<BasicValueEnum<'ctx>, CodegenError> {
        // Expressions typically have a type expected for them, typically inferred from something like a
        // variable declaration.
        let value_type = self.r#type.as_ref().ok_or(CodegenError::internal_error(
            "Integer literal was missing a type. Possible typechecker bug?".to_owned(),
            None,
        ))?;

        let llvm_type = value_type.resolve_value_type(codegen);
        if !llvm_type.is_int_type() {
            return Err(CodegenError::internal_error(
                format!("Unsupported value type in integer literal: {:?}", value_type.kind),
                value_type.location,
            ));
        }

        Ok(llvm_type
            .into_int_type()
            .const_int(self.value, false)
            .as_basic_value_enum())
    }
}

impl ExpressionCodegen for IdentifierReferenceNode {
    fn codegen<'ctx>(&self, codegen: &mut Codegen<'ctx>) -> Result<BasicValueEnum<'ctx>, CodegenError> {
        #[rustfmt::skip]
        let function_scope = codegen.context.function_scope.as_ref()
            .ok_or(CodegenError::internal_error(
                "Unable to reference a variable outside of a function block".to_owned(),
                None,
            ))?;

        let pointer = match function_scope.variables.get(&self.name) {
            Some(value) => value,
            None => panic!("Undeclared variable? {}", self.name),
        };

        let value_type = self.r#type.as_ref().ok_or(CodegenError::internal_error(
            "Identifier reference was missing a type. Possible typechecker bug?".to_owned(),
            None,
        ))?;

        codegen
            .llvm_builder
            .build_load(value_type.resolve_value_type(codegen), *pointer, &self.name)
            .map_err(|error| CodegenError::internal_error(error.to_string(), None))
    }
}
