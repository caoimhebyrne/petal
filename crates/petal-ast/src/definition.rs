use petal_span::Span;

use crate::statement::Block;

/// The different kinds of top-level definitions in the language.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Definition {
    Function(FunctionDefinition),
}

impl Definition {
    /// Return the [`Span`] that this [`Definition`] covers.
    pub fn span(&self) -> Span {
        match self {
            Definition::Function(function_definition) => function_definition.span,
        }
    }
}

/// A function definition.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FunctionDefinition {
    /// The name of the function.
    pub name: String,

    /// The return type of the function.
    pub return_type_name: Option<String>,

    /// The body of the function.
    pub body: Block,

    /// The [`Span`] that the function definition covers.
    pub span: Span,
}
