use crate::{
    ast::expression::{
        Expression,
        ExpressionKind,
        binary_operation::BinaryOperation,
        function_call::FunctionCall,
    },
    core::span::Span,
    typechecker::{
        Typechecker,
        error::{
            TypecheckerError,
            TypecheckerErrorKind,
        },
        r#type::Type,
    },
};

impl Typechecker {
    /// Checks and resolves the type of the provided [`Expression`].
    pub(crate) fn check_expression(&mut self, expression: &mut Expression) -> Result<Type, TypecheckerError> {
        let r#type = match &mut expression.kind {
            ExpressionKind::NumberLiteral(value) => Typechecker::check_number_literal(value, expression.span),

            ExpressionKind::BinaryOperation(binary_operation) => {
                self.check_binary_operation(binary_operation, expression.span)
            }

            ExpressionKind::FunctionCall(function_call) => self.check_function_call(function_call, expression.span),

            ExpressionKind::IdentifierReference(name) => self.check_identifier_reference(name, expression.span),
        }?;

        Ok(r#type)
    }

    /// Checks and resolves the type of the provided number literal.
    fn check_number_literal(_value: &f64, _span: Span) -> Result<Type, TypecheckerError> {
        // TODO: Use the context of the check to infer the type.
        //       e.g: If the checker expects an `i32`, and the literal supports that type, then we should use that.
        //       For now, all integer literals are i32.
        Ok(Type::SignedInteger(32))
    }

    /// Checks and resolves the type of the provided [`BinaryOperation`].
    fn check_binary_operation(
        &mut self,
        binary_operation: &mut BinaryOperation,
        span: Span,
    ) -> Result<Type, TypecheckerError> {
        // Types on both sides of the operation must be resolvable.
        let left = self.check_expression(&mut binary_operation.left)?;
        let right = self.check_expression(&mut binary_operation.right)?;

        // Both of the types must be the same. If they are not, then we must error.
        if left != right {
            return Err(TypecheckerErrorKind::IncompatibleBinaryOperationTypes { left, right }.at(span));
        }

        // TODO: Check if the operation is supported on the type. Some types do not support certain binary operations.
        //       To be safe, we whitelist integers.
        if matches!(left, Type::SignedInteger(_)) || matches!(left, Type::UnsignedInteger(_)) {
            return Ok(left);
        }

        Err(TypecheckerErrorKind::BinaryOperationNotSupported(left).at(span))
    }

    /// Checks and resolves the type of the provided [`FunctionCall`].
    fn check_function_call(&mut self, function_call: &mut FunctionCall, span: Span) -> Result<Type, TypecheckerError> {
        // The arguments in the function call must be resolved.
        let arguments = function_call
            .arguments
            .iter_mut()
            .map(|it| self.check_expression(it))
            .collect::<Result<Vec<Type>, TypecheckerError>>()?;

        // The function must have been declared already.
        let checked_function = self.get_checked_function(&function_call.name, span)?;

        // The parameters to the function must match the arguments of the function call.
        let parameters: Vec<Type> = checked_function.parameters.iter().map(|it| it.r#type).collect();

        // If the function call's arguments does not match the function's parameters, then we must not permit it.
        if arguments != parameters {
            return Err(TypecheckerErrorKind::InvalidFunctionCall {
                name: function_call.name.clone(),
                parameters,
                arguments,
            }
            .at(span));
        }

        Ok(checked_function.return_type)
    }

    /// Checks and resolves the type of the provided identifier reference.
    fn check_identifier_reference(&mut self, name: &str, span: Span) -> Result<Type, TypecheckerError> {
        self.get_variable(name, span).copied()
    }
}
