use petal_ast::{node::FunctionCall, statement::function_declaration::FunctionModifier};
use petal_core::{error::Result, source_span::SourceSpan, r#type::ResolvedType};

/// This module contains implementations of [Typecheck] for various expression kinds.
pub(crate) mod expression;

/// This module contains implementations of [Typecheck] for various statement kinds.
pub(crate) mod statement;

/// This module contains implementations of [Typecheck] for various top-level statement kinds.
pub(crate) mod top_level_statement;

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

impl<'a> Typecheck<'a> for FunctionCall {
    fn typecheck(
        &mut self,
        typechecker: &mut Typechecker<'a>,
        _expected_type: Option<&ResolvedType>,
        span: SourceSpan,
    ) -> Result<ResolvedType> {
        // A function must exist with the same name.
        let function = typechecker.context.get_function(&self.name, span)?.clone();

        // If the function was declared in a different module, then we must throw an error.
        if function.module_id != *typechecker.context.module_id(span)?
            && !function.modifiers.contains(&FunctionModifier::Public)
        {
            let function_name = typechecker
                .string_intern_pool
                .resolve_reference_or_err(&self.name, span)?;

            return TypecheckerError::cross_module_reference(function_name, span).into();
        }

        // The arguments passed must equal the amount of parameters.
        if function.parameters.len() != self.arguments.len() {
            return TypecheckerError::incorrect_number_of_arguments(
                function.parameters.len(),
                self.arguments.len(),
                span,
            )
            .into();
        }

        // The type of each argument must equal the type of its corresponding parameter.
        for (index, argument) in self.arguments.iter_mut().enumerate() {
            let parameter_type = function.parameters.get(index).expect("Failed to access nth parameter?");
            let argument_type = typechecker.check_expression(argument, Some(parameter_type))?;

            if argument_type != *parameter_type {
                return TypecheckerError::expected_type(parameter_type.clone(), argument_type, argument.span).into();
            }
        }

        Ok(function.return_type.clone())
    }
}
