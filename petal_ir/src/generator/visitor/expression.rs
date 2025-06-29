use crate::{
    error::{IRError, IRResult},
    generator::IRGenerator,
    value::{Value, ValueType, binary_operation::Operand},
};
use petal_core::ast::{self, node::operator::Operation};

/// A visitor for an AST expression.
/// This converts a [Expression] into an IR [Value].
pub trait ExpressionVisitor {
    fn visit(&self, generator: &mut IRGenerator) -> IRResult<Value>;
}

impl ExpressionVisitor for ast::node::expression::IntegerLiteral {
    fn visit(&self, _generator: &mut IRGenerator) -> IRResult<Value> {
        let value_type = match &self.expected_type {
            Some(value) => value.clone().into(),
            None => return Err(IRError::missing_type_information(self.node.location)),
        };

        Ok(Value::new_integer_literal(self.value, value_type))
    }
}

impl ExpressionVisitor for ast::node::expression::StringLiteral {
    fn visit(&self, generator: &mut IRGenerator) -> IRResult<Value> {
        // A string literal must occur in a function scope.
        let function_scope = generator.function_scope(self.node.location)?;

        // The index of the item is the current length of the data section.
        let index = function_scope.data.len();
        function_scope.data.push(self.value.clone().into_bytes());

        Ok(Value::new_data_section_reference(index))
    }
}

impl ExpressionVisitor for ast::node::expression::IdentifierReference {
    fn visit(&self, generator: &mut IRGenerator) -> IRResult<Value> {
        // An identifier reference must occur in a function scope.
        let function_scope = generator.function_scope(self.node.location)?;

        let index = match function_scope.locals.iter().position(|it| it.name == self.name) {
            Some(value) => value,
            _ => return Err(IRError::undefined_identifier(self.node.location)),
        };

        let expected_value_type = match &self.expected_type {
            Some(value) => value.clone().into(),
            None => return Err(IRError::missing_type_information(self.node.location)),
        };

        let value_type = if self.is_reference {
            ValueType::Reference
        } else {
            expected_value_type
        };

        Ok(Value::new_local_reference(index, value_type))
    }
}

impl ExpressionVisitor for ast::node::expression::BinaryOperation {
    fn visit(&self, generator: &mut IRGenerator) -> IRResult<Value> {
        let lhs = generator.visit_expression(&self.left)?;
        let rhs = generator.visit_expression(&self.right)?;

        let value_type = match &self.expected_type {
            Some(value) => value.clone().into(),
            None => return Err(IRError::missing_type_information(self.node.location)),
        };

        let operand = match self.operation {
            Operation::Add => Operand::Add,
            Operation::Divide => Operand::Divide,
            Operation::Multiply => Operand::Multiply,
            Operation::Subtract => Operand::Subtract,
        };

        Ok(Value::new_binary_operation(lhs, rhs, operand, value_type))
    }
}

impl ExpressionVisitor for ast::node::expression::FunctionCall {
    fn visit(&self, generator: &mut IRGenerator) -> IRResult<Value> {
        let arguments = self
            .arguments
            .iter()
            .map(|it| generator.visit_expression(it))
            .collect::<IRResult<Vec<Value>>>()?;

        let value_type = match &self.expected_type {
            Some(value) => value.clone().into(),
            None => return Err(IRError::missing_type_information(self.node.location)),
        };

        Ok(Value::new_function_call(self.name.clone(), arguments, value_type))
    }
}
