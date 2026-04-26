use crate::{
    ast::expression::{
        Expression,
        ExpressionKind,
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
}
