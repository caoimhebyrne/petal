use inkwell::{
    types::BasicTypeEnum,
    values::{BasicValue, BasicValueEnum},
};

use crate::ast::node::kind::IntegerLiteralNode;

use super::Codegen;

pub trait ExpressionCodegen {
    fn codegen<'ctx>(
        &self,
        codegen: &Codegen<'ctx>,
        expected_type: Option<BasicTypeEnum<'ctx>>,
    ) -> BasicValueEnum<'ctx>;
}

impl ExpressionCodegen for IntegerLiteralNode {
    fn codegen<'ctx>(
        &self,
        codegen: &Codegen<'ctx>,
        expected_type: Option<BasicTypeEnum<'ctx>>,
    ) -> BasicValueEnum<'ctx> {
        // Expressions typically have a type expected for them, typically inferred from something like a
        // variable declaration.
        let value_type = if let Some(the_type) = expected_type {
            if the_type.is_int_type() {
                the_type.into_int_type()
            } else {
                panic!(
                    "A non-integer type was expected when generating an integer literal. Expected: {}",
                    the_type.to_string()
                )
            }
        } else {
            // If no type is expected, let's just assume that it is an i32.
            codegen.context.i32_type()
        };

        value_type
            .const_int(self.value, false)
            .as_basic_value_enum()
    }
}
