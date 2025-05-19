use super::{r#type::TypeCodegen, Codegen};
use crate::ast::node::kind::{IdentifierReferenceNode, IntegerLiteralNode};
use inkwell::values::{BasicValue, BasicValueEnum};

pub trait ExpressionCodegen {
    fn codegen<'ctx>(&self, codegen: &mut Codegen<'ctx>) -> BasicValueEnum<'ctx>;
}

impl ExpressionCodegen for IntegerLiteralNode {
    fn codegen<'ctx>(&self, codegen: &mut Codegen<'ctx>) -> BasicValueEnum<'ctx> {
        // Expressions typically have a type expected for them, typically inferred from something like a
        // variable declaration.
        let value_type = self
            .r#type
            .clone()
            .map(|it| it.resolve_value_type(codegen))
            .expect("Unsupported value type for integer literal");

        if !value_type.is_int_type() {
            panic!("Unsupported value type '{:?}' in integer literal", value_type)
        }

        value_type
            .into_int_type()
            .const_int(self.value, false)
            .as_basic_value_enum()
    }
}

impl ExpressionCodegen for IdentifierReferenceNode {
    fn codegen<'ctx>(&self, codegen: &mut Codegen<'ctx>) -> BasicValueEnum<'ctx> {
        let function_scope = match &codegen.context.function_scope {
            Some(value) => value,
            None => panic!("Identifier reference outside of function scope?"),
        };

        let pointer = match function_scope.variables.get(&self.name) {
            Some(value) => value,
            None => panic!("Undeclared variable? {}", self.name),
        };

        let value_type = self
            .r#type
            .clone()
            .map(|it| it.resolve_value_type(codegen))
            .expect("Unsupported value type for identifier reference");

        codegen
            .llvm_builder
            .build_load(value_type, *pointer, &self.name)
            .expect("Failed to build value for load")
    }
}
