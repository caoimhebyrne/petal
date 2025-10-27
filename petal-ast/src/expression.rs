use petal_core::source_span::SourceSpan;

use crate::r#type::Type;

/// An expression can be seen as an action that can return a value.
#[derive(Debug, Clone, PartialEq)]
pub struct Expression {
    /// The kind of expression that this is.
    pub kind: ExpressionKind,

    /// The type of the value that this expression produces.
    pub r#type: Option<Type>,

    /// The span within the source code that this expression was defined at.
    pub span: SourceSpan,
}

impl Expression {
    pub fn new(kind: ExpressionKind, span: SourceSpan) -> Self {
        Expression {
            kind,
            r#type: None,
            span,
        }
    }
}

/// The different kinds of expressions that exist.
#[derive(Debug, Clone, PartialEq)]
pub enum ExpressionKind {
    // An integer literal, e.g: `12345`.
    IntegerLiteral(u64),
}
