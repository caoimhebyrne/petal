use petal_ast::statement::function_call::FunctionCall;
use petal_core::{error::Result, source_span::SourceSpan, r#type::ResolvedType};

/// This module contains implementations of [Typecheck] for various expression kinds.
pub(crate) mod expression;

/// This module contains implementations of [Typecheck] for various statement kinds.
pub(crate) mod statement;

use crate::{Typechecker, error::TypecheckerError};

/// The [Typecheck] trait is a trait that all types that wish to be type-checkable can implement.
///
/// See also: [Typechecker::check_statement].
pub trait Typecheck<'a> {
    /// Resolves any and all types referenced by the node and ensures that they are valid. The caller is responsible for
    /// mutating the node to contain fully-resolved type information.
    ///
    /// An [expected_type] is provided, this may be used as a hint by the caller for type inference (primarily used by
    /// integer literals).
    ///
    /// Returns:
    /// The [Type] that this node produces. If the node does not result in a type, then [Type::void] should be returned.
    fn typecheck(
        &mut self,
        typechecker: &mut Typechecker<'a>,
        expected_type: Option<&ResolvedType>,
        span: SourceSpan,
    ) -> Result<ResolvedType>;
}

/// A function call is both a statement and expression, so it doesn't belong in either of the submodules.
impl<'a> Typecheck<'a> for FunctionCall {
    fn typecheck(
        &mut self,
        typechecker: &mut Typechecker<'a>,
        _expected_type: Option<&ResolvedType>,
        span: SourceSpan,
    ) -> Result<ResolvedType> {
        // FIXME: I don't like this clone, but we need to do it because of the borrow checker.
        let function = typechecker.context.get_function(&self.name_reference, span).cloned()?;

        // The number of arguments must be equal to the number of parameters in the function.
        let is_variadic = function.parameters.last() == Some(&ResolvedType::Variadic);
        if is_variadic {
            if self.arguments.len() < function.parameters.len() {
                return TypecheckerError::incorrect_number_of_arguments(
                    function.parameters.len(),
                    self.arguments.len(),
                    span,
                )
                .into();
            }
        } else if function.parameters.len() != self.arguments.len() {
            return TypecheckerError::incorrect_number_of_arguments(
                function.parameters.len(),
                self.arguments.len(),
                span,
            )
            .into();
        }

        for (index, argument) in self.arguments.iter_mut().enumerate() {
            // NOTE: The `unwrap` is safe here, we just verified that the function call had the correct amount of
            // arguments.
            let parameter_type = function
                .parameters
                .get(index.clamp(0, function.parameters.len() - 1))
                .unwrap();

            let argument_type = typechecker.check_expression(argument, Some(parameter_type))?;

            if *parameter_type != ResolvedType::Variadic && *parameter_type != argument_type {
                return TypecheckerError::expected_type(parameter_type.clone(), argument_type, argument.span).into();
            }
        }

        // The value type is always going to be the return type of the function that is being called.
        Ok(function.return_type)
    }
}
