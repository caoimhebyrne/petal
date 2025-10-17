use crate::{ast::expression::Expression, core::source_span::SourceSpan};

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
    /// FIXME: A `String` is not optimal here. It would make sense to implement some string intering in the future:
    /// https://en.wikipedia.org/wiki/String_interning
    pub name: String,

    /// The value being assigned to the variable.
    pub value: Expression,
}

impl VariableDeclaration {
    /// Creates a new [VariableDeclaration] with a [name] and [value].
    pub fn new(name: String, value: Expression) -> VariableDeclaration {
        VariableDeclaration { name, value }
    }
}
