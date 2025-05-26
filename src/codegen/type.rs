use super::Codegen;
use crate::typechecker::r#type::{Type, kind::TypeKind};
use core::panic;
use inkwell::{
    AddressSpace,
    types::{BasicMetadataTypeEnum, BasicType, BasicTypeEnum, FunctionType},
};

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
                .llvm_context
                .custom_width_int_type((*width).into())
                .fn_type(param_types, is_var_args),

            TypeKind::Reference(_) => codegen
                .llvm_context
                .ptr_type(AddressSpace::default())
                .fn_type(param_types, is_var_args),

            TypeKind::Boolean => codegen.llvm_context.bool_type().fn_type(param_types, is_var_args),

            TypeKind::Void => codegen.llvm_context.void_type().fn_type(param_types, is_var_args),

            TypeKind::Unresolved(name) => panic!("Unable to resolve codegen type for unresolved type: '{}'", name),
        }
    }

    fn resolve_value_type<'ctx>(&self, codegen: &Codegen<'ctx>) -> BasicTypeEnum<'ctx> {
        match &self.kind {
            TypeKind::Integer(width) => codegen
                .llvm_context
                .custom_width_int_type((*width).into())
                .as_basic_type_enum(),

            TypeKind::Reference(_) => codegen
                .llvm_context
                .ptr_type(AddressSpace::default())
                .as_basic_type_enum(),

            TypeKind::Boolean => codegen.llvm_context.bool_type().as_basic_type_enum(),

            TypeKind::Void => panic!("Unable to use `void` as a value type"),

            TypeKind::Unresolved(name) => panic!("Unable to resolve codegen type for unresolved type: '{}'", name),
        }
    }
}
