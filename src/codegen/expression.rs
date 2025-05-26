use super::{Codegen, error::CodegenError, r#type::TypeCodegen};
use crate::ast::node::{
    expression::{
        BinaryComparison, BinaryOperation, BooleanLiteral, FunctionCall, IdentifierReference, IntegerLiteral,
        StringLiteral,
    },
    operator::{Comparison, Operation},
};
use inkwell::{
    IntPredicate,
    values::{BasicValue, BasicValueEnum, InstructionOpcode},
};

pub trait ExpressionCodegen {
    fn codegen<'ctx>(&self, codegen: &mut Codegen<'ctx>) -> Result<BasicValueEnum<'ctx>, CodegenError>;
}

impl ExpressionCodegen for IntegerLiteral {
    fn codegen<'ctx>(&self, codegen: &mut Codegen<'ctx>) -> Result<BasicValueEnum<'ctx>, CodegenError> {
        // Expressions typically have a type expected for them, typically inferred from something like a
        // variable declaration.
        let value_type = self.expected_type.as_ref().ok_or(CodegenError::internal_error(
            "Integer literal was missing a type. Possible typechecker bug?".to_owned(),
            None,
        ))?;

        let llvm_type = value_type.resolve_value_type(codegen);
        if !llvm_type.is_int_type() {
            return Err(CodegenError::internal_error(
                format!("Unsupported value type in integer literal: {:?}", value_type.kind),
                Some(value_type.location),
            ));
        }

        Ok(llvm_type
            .into_int_type()
            .const_int(self.value, false)
            .as_basic_value_enum())
    }
}

impl ExpressionCodegen for StringLiteral {
    fn codegen<'ctx>(&self, codegen: &mut Codegen<'ctx>) -> Result<BasicValueEnum<'ctx>, CodegenError> {
        codegen
            .llvm_builder
            .build_global_string_ptr(&self.value, "string_literal")
            .map(|it| it.as_basic_value_enum())
            .map_err(|error| CodegenError::internal_error(error.to_string(), None))
    }
}

impl ExpressionCodegen for IdentifierReference {
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

        let value_type = self.expected_type.as_ref().ok_or(CodegenError::internal_error(
            "Identifier reference was missing a type. Possible typechecker bug?".to_owned(),
            None,
        ))?;

        if self.is_reference {
            return Ok(pointer.as_basic_value_enum());
        }

        codegen
            .llvm_builder
            .build_load(value_type.resolve_value_type(codegen), *pointer, &self.name)
            .map_err(|error| CodegenError::internal_error(error.to_string(), None))
    }
}

impl ExpressionCodegen for BinaryOperation {
    fn codegen<'ctx>(&self, codegen: &mut Codegen<'ctx>) -> Result<BasicValueEnum<'ctx>, CodegenError> {
        let left = Codegen::visit_expression(codegen, &self.left)?;
        let right = Codegen::visit_expression(codegen, &self.right)?;

        let operation = match self.operation {
            Operation::Add => InstructionOpcode::Add,
            Operation::Subtract => InstructionOpcode::Sub,
            Operation::Multiply => InstructionOpcode::Mul,
            Operation::Divide => InstructionOpcode::SDiv,
        };

        codegen
            .llvm_builder
            .build_binop(operation, left, right, "binop")
            .map_err(|error| CodegenError::internal_error(error.to_string(), None))
    }
}

impl ExpressionCodegen for BinaryComparison {
    fn codegen<'ctx>(&self, codegen: &mut Codegen<'ctx>) -> Result<BasicValueEnum<'ctx>, CodegenError> {
        let left = Codegen::visit_expression(codegen, &self.left)?;
        let right = Codegen::visit_expression(codegen, &self.right)?;

        let operation = match self.comparison {
            Comparison::GreaterThan => IntPredicate::SGT,
            Comparison::LessThan => IntPredicate::SLT,
        };

        codegen
            .llvm_builder
            .build_int_compare(operation, left.into_int_value(), right.into_int_value(), "compare")
            .map(|it| it.as_basic_value_enum())
            .map_err(|error| CodegenError::internal_error(error.to_string(), None))
    }
}

impl ExpressionCodegen for FunctionCall {
    fn codegen<'ctx>(&self, codegen: &mut Codegen<'ctx>) -> Result<BasicValueEnum<'ctx>, CodegenError> {
        let function = codegen
            .llvm_module
            .get_function(&self.name)
            .ok_or(CodegenError::internal_error(
                format!("Failed to find a function with the name '{}'", self.name),
                None,
            ))?;

        let mut arguments = vec![];
        for argument in &self.arguments {
            arguments.push(Codegen::visit_expression(codegen, argument)?.into());
        }

        codegen
            .llvm_builder
            .build_call(function, &arguments, &self.name)
            .map(|it| it.try_as_basic_value().expect_left("value was right"))
            .map_err(|error| CodegenError::internal_error(error.to_string(), None))
    }
}

impl ExpressionCodegen for BooleanLiteral {
    fn codegen<'ctx>(&self, codegen: &mut Codegen<'ctx>) -> Result<BasicValueEnum<'ctx>, CodegenError> {
        let llvm_type = codegen.llvm_context.bool_type();

        println!("{:?}", self.value);

        Ok(llvm_type
            .const_int(if self.value { 1 } else { 0 }, false)
            .as_basic_value_enum())
    }
}
