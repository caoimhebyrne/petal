use petal_core::{source_span::SourceSpan, string_intern::StringReference, r#type::pool::TypeId};

use crate::statement::{Statement, StatementKind, r#return::ReturnStatement};

/// A parameter defined in a [FunctionDeclaration].
#[derive(Debug, Clone, PartialEq)]
pub struct FunctionParameter {
    /// The identifier of the parameter.
    pub name_reference: StringReference,

    /// The type of the parameter's expected value.
    pub value_type: TypeId,

    /// The span within the source code that the paramter was defined at.
    pub span: SourceSpan,
}

impl FunctionParameter {
    pub fn new(name_reference: StringReference, value_type: TypeId, span: SourceSpan) -> Self {
        FunctionParameter {
            name_reference,
            value_type,
            span,
        }
    }
}

/// A function declaration statement, e.g. `func <name>() { <body> }`
#[derive(Debug, Clone, PartialEq)]
pub struct FunctionDeclaration {
    /// The name of the function.
    pub name_reference: StringReference,

    /// The parameters of the function.
    pub parameters: Vec<FunctionParameter>,

    /// The return type of the function.
    pub return_type: TypeId,

    /// The body of the function.
    pub body: Vec<Statement>,
}

impl FunctionDeclaration {
    /// Creates a new [FunctionDeclaration] with a [name_reference] and [body]
    pub fn new(
        name_reference: StringReference,
        parameters: Vec<FunctionParameter>,
        return_type: TypeId,
        body: Vec<Statement>,
    ) -> Self {
        FunctionDeclaration {
            name_reference,
            parameters,
            return_type,
            body,
        }
    }

    /// Inserts an implicit return statement at the end of this [FunctionDeclaration]'s body.
    pub fn insert_implicit_return_void(&mut self, span: SourceSpan) {
        self.body.push(Statement {
            kind: StatementKind::ReturnStatement(ReturnStatement::new(None)),
            span: self.body.last().map(|it| it.span).unwrap_or(span),
        });
    }
}

/// Allows `.into()` to be called on a [FunctionDeclaration] to turn it into a [StatementKind].
impl From<FunctionDeclaration> for StatementKind {
    fn from(value: FunctionDeclaration) -> Self {
        StatementKind::FunctionDeclaration(value)
    }
}
