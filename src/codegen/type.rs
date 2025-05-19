use super::Codegen;
use crate::typechecker::r#type::{Type, kind::TypeKind};
use core::panic;
use inkwell::types::{BasicMetadataTypeEnum, BasicType, BasicTypeEnum, FunctionType};

pub trait TypeCodegen {
    fn resolve_fn_type<'ctx>(
        &self,
        codegen: &Codegen<'ctx>,
        param_types: &[BasicMetadataTypeEnum<'ctx>],
        is_var_args: bool,
    ) -> FunctionType<'ctx>;

    fn resolve_value_type<'ctx>(&self, codegen: &Codegen<'ctx>) -> BasicTypeEnum<'ctx>;
}

impl TypeCodegen for Type {
    fn resolve_fn_type<'ctx>(
        &self,
        codegen: &Codegen<'ctx>,
        param_types: &[BasicMetadataTypeEnum<'ctx>],
        is_var_args: bool,
    ) -> FunctionType<'ctx> {
        match &self.kind {
            TypeKind::Integer(width) => codegen
                .context
                .custom_width_int_type((*width).into())
                .fn_type(param_types, is_var_args),

            TypeKind::Void => codegen
                .context
                .void_type()
                .fn_type(param_types, is_var_args),

            TypeKind::Unresolved(name) => panic!(
                "Unable to resolve codegen type for unresolved type: '{}'",
                name
            ),
        }
    }

    fn resolve_value_type<'ctx>(&self, codegen: &Codegen<'ctx>) -> BasicTypeEnum<'ctx> {
        match &self.kind {
            TypeKind::Integer(width) => codegen
                .context
                .custom_width_int_type((*width).into())
                .as_basic_type_enum(),

            TypeKind::Void => panic!("Unable to use `void` as a value type"),

            TypeKind::Unresolved(name) => panic!(
                "Unable to resolve codegen type for unresolved type: '{}'",
                name
            ),
        }
    }
}
