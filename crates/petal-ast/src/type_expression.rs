use petal_span::Span;

/// A reference to a named type in the source code.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeExpression {
    /// The name of the type being referenced.
    pub name: String,

    /// The location within the source code that the reference is at.
    pub span: Span,
}
