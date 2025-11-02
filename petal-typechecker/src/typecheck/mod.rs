use petal_ast::{statement::function_call::FunctionCall, r#type::Type};
use petal_core::{error::Result, source_span::SourceSpan};

/// This module contains implementations of [Typecheck] for various expression kinds.
pub(crate) mod expression;

/// This module contains implementations of [Typecheck] for various statement kinds.
pub(crate) mod statement;

use crate::Typechecker;

/// The [Typecheck] trait is a trait that all types that wish to be type-checkable can implement.
///
/// See also: [Typechecker::check_statement].
pub trait Typecheck<'a> {
    /// Resolves any and all types referenced by the node and ensures that they are valid. The caller is responsible for
    /// mutating the node to contain fully-resolved type information.
    ///
    /// Returns:
    /// The [Type] that this node produces. If the node does not result in a type, then [Type::void] should be returned.
    fn typecheck(&mut self, typechecker: &mut Typechecker<'a>, span: SourceSpan) -> Result<Type>;
}

/// A function call is both a statement and expression, so it doesn't belong in either of the submodules.
impl<'a> Typecheck<'a> for FunctionCall {
    fn typecheck(&mut self, typechecker: &mut Typechecker<'a>, span: SourceSpan) -> Result<Type> {
        // The value type is always going to be the return type of the function that is being called.
        let function = typechecker.context.get_function(&self.name_reference, span)?;
        Ok(function.return_type)
    }
}
