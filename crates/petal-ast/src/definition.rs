use petal_span::Span;

use crate::{TypeExpression, statement::Block};

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
    /// A unique ID for the function definition.
    pub id: FunctionId,

    /// The name of the function.
    pub name: String,

    /// The return type of the function.
    pub return_type: Option<TypeExpression>,

    /// The body of the function.
    pub body: Block,

    /// The [`Span`] that the function definition covers.
    pub span: Span,
}

/// A unique identifier assigned to a function.
///
/// IDs are allocated by the parser, starting from zero for each parse, so they can be used to index into a table
/// sized by [`crate::ParseResult::function_count`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FunctionId(pub(crate) u32);

impl FunctionId {
    /// Return the index of this ID, for use in tables indexed by [`FunctionId`].
    pub const fn index(self) -> usize {
        self.0 as usize
    }
}
