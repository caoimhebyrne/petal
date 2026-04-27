use crate::{
    ast::expression::{
        Expression,
        ExpressionKind,
        binary_operation::{
            BinaryOperand,
            BinaryOperation,
        },
        function_call::FunctionCall,
    },
    backend::c::{
        CBackend,
        error::CBackendError,
    },
    core::span::Span,
};

impl CBackend {
    /// Compiles an expression into C code.
    pub fn compile_expression(expression: &Expression) -> Result<String, CBackendError> {
        match &expression.kind {
            ExpressionKind::FunctionCall(function_call) => {
                CBackend::compile_function_call(function_call, expression.span)
            }

            ExpressionKind::BinaryOperation(binary_operation) => {
                CBackend::compile_binary_operation(binary_operation, expression.span)
            }

            ExpressionKind::NumberLiteral(value) => CBackend::compile_number_literal(value, expression.span),
            ExpressionKind::IdentifierReference(name) => CBackend::compile_identifier_reference(name, expression.span),
        }
    }

    /// Compiles a number literal expression into C code.
    pub fn compile_number_literal(value: &f64, _span: Span) -> Result<String, CBackendError> {
        Ok(value.to_string())
    }

    /// Compiles an identifier reference expression into C code.
    pub fn compile_identifier_reference(value: &String, _span: Span) -> Result<String, CBackendError> {
        Ok(value.to_string())
    }

    /// Compiles a function call expression into C code.
    pub fn compile_function_call(function_call: &FunctionCall, _span: Span) -> Result<String, CBackendError> {
        let arguments = &function_call
            .arguments
            .iter()
            .map(|it| CBackend::compile_expression(&it.value))
            .collect::<Result<Vec<String>, CBackendError>>()?
            .join(", ");

        Ok(format!("{}({arguments})", function_call.name))
    }

    /// Compiles a binary operation expression into C code.
    pub fn compile_binary_operation(binary_operation: &BinaryOperation, _span: Span) -> Result<String, CBackendError> {
        let left = CBackend::compile_expression(&binary_operation.left)?;

        let right = CBackend::compile_expression(&binary_operation.right)?;

        let operand = match binary_operation.operand {
            BinaryOperand::Add => "+",
            BinaryOperand::Subtract => "-",
            BinaryOperand::Multiply => "*",
            BinaryOperand::Divide => "/",
        };

        Ok(format!("{left} {operand} {right}"))
    }
}
