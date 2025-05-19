use super::{Codegen, r#type::TypeCodegen};
use crate::ast::node::kind::IntegerLiteralNode;
use inkwell::values::{BasicValue, BasicValueEnum};

pub trait ExpressionCodegen {
    fn codegen<'ctx>(&self, codegen: &Codegen<'ctx>) -> BasicValueEnum<'ctx>;
}

impl ExpressionCodegen for IntegerLiteralNode {
    fn codegen<'ctx>(&self, codegen: &Codegen<'ctx>) -> BasicValueEnum<'ctx> {
        // Expressions typically have a type expected for them, typically inferred from something like a
        // variable declaration.
        let value_type = self
            .r#type
            .clone()
            .map(|it| it.resolve_value_type(codegen))
            .expect("Unsupported value type for integer literal");

        if !value_type.is_int_type() {
            panic!(
                "Unsupported value type '{:?}' in integer literal",
                value_type
            )
        }

        value_type
            .into_int_type()
            .const_int(self.value, false)
            .as_basic_value_enum()
    }
}
