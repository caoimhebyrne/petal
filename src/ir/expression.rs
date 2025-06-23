use crate::{
    ast::{
        self,
        node::{expression::IdentifierReference, operator::Operation},
    },
    ir::{
        BinaryOperation, FunctionCall, IntegerLiteral, Operand, Value, VariableReference,
        context::Context,
        generator::{IRResult, IntermediateRepresentation},
    },
};

/// Visits an expression inthe AST, converting it to a [Value].
pub(crate) trait ExpressionVisitor {
    fn visit(&self, context: &mut Context) -> IRResult<Value>;
}

impl ExpressionVisitor for ast::node::expression::IntegerLiteral {
    fn visit(&self, _context: &mut Context) -> IRResult<Value> {
        Ok(Value::IntegerLiteral(IntegerLiteral {
            value: self.value.try_into().unwrap(),
        }))
    }
}

impl ExpressionVisitor for IdentifierReference {
    fn visit(&self, context: &mut Context) -> IRResult<Value> {
        let variable_index = context.function_scope(self.node)?.find_variable_index(&self.name);
        Ok(Value::VariableReference(VariableReference { variable_index }))
    }
}

impl ExpressionVisitor for ast::node::expression::BinaryOperation {
    fn visit(&self, context: &mut Context) -> IRResult<Value> {
        Ok(Value::BinaryOperation(BinaryOperation {
            left: Box::new(IntermediateRepresentation::visit_expression(context, &self.left)?),
            right: Box::new(IntermediateRepresentation::visit_expression(context, &self.right)?),

            operand: match self.operation {
                Operation::Add => Operand::Add,
                Operation::Subtract => Operand::Subtract,
                Operation::Multiply => Operand::Multiply,
                Operation::Divide => Operand::Divide,
            },
        }))
    }
}

impl ExpressionVisitor for ast::node::expression::FunctionCall {
    fn visit(&self, context: &mut Context) -> IRResult<Value> {
        let arguments = self
            .arguments
            .iter()
            .map(|it| IntermediateRepresentation::visit_expression(context, it))
            .collect::<IRResult<Vec<Value>>>()?;

        Ok(Value::FunctionCall(FunctionCall {
            name: self.name.clone(),
            arguments,
        }))
    }
}
