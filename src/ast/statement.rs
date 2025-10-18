use crate::{
    ast::expression::Expression,
    core::{source_span::SourceSpan, string_intern::StringReference},
};

/// A statement can be seen as an action that does not return a value.
#[derive(Debug, Clone, PartialEq)]
pub struct Statement {
    /// The kind of statement that this is.
    pub kind: StatementKind,

    /// The span within the source code that this statement was defined at.
    pub span: SourceSpan,
}

/// The different kinds of statements that exist.
#[derive(Debug, Clone, PartialEq)]
pub enum StatementKind {
    /// A variable declaration statement, e.g: `let <identifier> = <expression>;`
    VariableDeclaration(VariableDeclaration),
}

/// A variable declaration statement, e.g: `let <identifier> = <expression>;`
#[derive(Debug, Clone, PartialEq)]
pub struct VariableDeclaration {
    /// The name of the variable being declared.
    pub identifier_reference: StringReference,

    /// The value being assigned to the variable.
    pub value: Expression,
}

impl VariableDeclaration {
    /// Creates a new [VariableDeclaration] with a [name] and [value].
    pub fn new(identifier_reference: StringReference, value: Expression) -> Self {
        VariableDeclaration {
            identifier_reference,
            value,
        }
    }
}

/// Allows `.into()` to be called on a [VariableDeclaration] to turn it into a [StatementKind].
impl From<VariableDeclaration> for StatementKind {
    fn from(value: VariableDeclaration) -> Self {
        StatementKind::VariableDeclaration(value)
    }
}
