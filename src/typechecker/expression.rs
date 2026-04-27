use crate::{
    ast::expression::{
        Expression,
        ExpressionKind,
        binary_operation::BinaryOperation,
        function_call::{
            FunctionCall,
            FunctionCallArgument,
        },
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
        // FIXME: I don't like the clone here.
        let checked_function = self.get_checked_function(&function_call.name, span).cloned()?;

        // The first check is easy, we just need to ensure that a sufficient number of arguments were passed in the
        // function call.
        if function_call.arguments.len() != checked_function.parameters.len() {
            return Err(TypecheckerErrorKind::FunctionCallArgumentSizeMismatch {
                name: function_call.name.clone(),
                expected: checked_function.parameters.len(),
                got: function_call.arguments.len(),
            }
            .at(span));
        }

        // We also must check that each function call argument has been supplied once, anything else is undefined behavior.
        for argument in &function_call.arguments {
            let occurrences = function_call.arguments.iter().filter(|it| it.name == argument.name).count();
            if occurrences > 1 {
                return Err(TypecheckerErrorKind::DuplicateFunctionCallArgument(argument.name.clone()).at(span));
            }
        }

        // FIXME: I don't know if this really belongs here, but it's the best place that I can put it for now. We
        //        are going to update the the function call to ensure that the Vec order of the arguments is
        //        the same order that the parameters are defined in.
        //
        //        This allows the codegen backend to just iterate over the arguments in order, without needing to
        //        check whether they match to the function's parameters.
        let mut ordered_arguments: Vec<FunctionCallArgument> = Vec::new();

        for parameter in &checked_function.parameters {
            let argument = function_call.arguments.iter_mut().find(|it| it.name == parameter.name).ok_or(
                TypecheckerErrorKind::MissingFunctionCallArgument {
                    function_name: function_call.name.clone(),
                    parameter_name: parameter.name.clone(),
                }
                .at(span),
            )?;

            let argument_type = self.check_expression(&mut argument.value)?;
            if argument_type != parameter.r#type {
                return Err(TypecheckerErrorKind::IncompatibleFunctionCallArgument {
                    parameter_name: parameter.name.clone(),
                    parameter_type: parameter.r#type,
                    argument_type,
                }
                .at(argument.span));
            }

            ordered_arguments.push(argument.clone());
        }

        function_call.arguments = ordered_arguments;

        // All arguments have been type checked. The result of this function call is the return type of the function.
        Ok(checked_function.return_type)
    }

    /// Checks and resolves the type of the provided identifier reference.
    fn check_identifier_reference(&mut self, name: &str, span: Span) -> Result<Type, TypecheckerError> {
        self.get_variable(name, span).copied()
    }
}
