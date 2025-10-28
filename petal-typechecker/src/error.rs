use std::fmt::Display;

use petal_ast::{
    expression::{Expression, ExpressionKind},
    statement::{Statement, StatementKind},
    r#type::{Type, TypeKind},
};
use petal_core::{
    error::{Error, ErrorKind},
    source_span::SourceSpan,
    string_intern::StringReference,
};

/// Represents the different kinds of errors that can be thrown during a typecheck.
#[derive(Debug, Clone, PartialEq)]
pub enum TypecheckerErrorKind {
    /// A string reference was encountered that could not be resolved.
    UnresolvableStringReference(StringReference),

    /// A type was referenced that could not be resolved.
    UnresolvableType(String),

    /// A statement was encountered that could not be typechecked.
    UnsupportedStatement(StatementKind),

    /// An expression was encountered that could not be typechecked.,
    UnsupportedExpression(ExpressionKind),

    /// The typechecker's context was missing (for whatever reason).
    MissingContext,

    /// A type was expected, but a different one was received.
    ExpectedType { expected: TypeKind, received: TypeKind },

    /// A value was not returned from a function block.
    MissingReturnStatement,

    /// A variable was already declared with the same name.
    DuplicateVariableDeclaration,

    /// A variable was referenced that was not declared.
    UndeclaredVariable(String),
}

impl TypecheckerErrorKind {
    pub fn unresolvable_string_reference(reference: StringReference, span: SourceSpan) -> Error {
        Error::new(TypecheckerErrorKind::UnresolvableStringReference(reference), span)
    }

    pub fn unresolvable_type(name: &str, span: SourceSpan) -> Error {
        Error::new(TypecheckerErrorKind::UnresolvableType(name.to_owned()), span)
    }

    pub fn unsupported_statement(statement: &Statement) -> Error {
        Error::new(
            TypecheckerErrorKind::UnsupportedStatement(statement.kind.clone()),
            statement.span,
        )
    }

    pub fn unsupported_expression(expression: &Expression) -> Error {
        Error::new(
            TypecheckerErrorKind::UnsupportedExpression(expression.kind.clone()),
            expression.span,
        )
    }

    pub fn missing_context(span: SourceSpan) -> Error {
        Error::new(TypecheckerErrorKind::MissingContext, span)
    }

    pub fn expected_type(expected: &Type, received: &Type) -> Error {
        Error::new(
            TypecheckerErrorKind::ExpectedType {
                expected: expected.kind,
                received: received.kind,
            },
            received.span,
        )
    }

    pub fn missing_return_statement(span: SourceSpan) -> Error {
        Error::new(TypecheckerErrorKind::MissingReturnStatement, span)
    }

    pub fn duplicate_variable_declaration(span: SourceSpan) -> Error {
        Error::new(TypecheckerErrorKind::DuplicateVariableDeclaration, span)
    }

    pub fn undeclared_variable(name: &str, span: SourceSpan) -> Error {
        Error::new(TypecheckerErrorKind::UndeclaredVariable(name.to_owned()), span)
    }
}

impl Display for TypecheckerErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TypecheckerErrorKind::UnresolvableStringReference(reference) => {
                write!(
                    f,
                    "Unable to resolve string reference: '{:?}', this is 100% a compiler bug",
                    reference
                )
            }

            TypecheckerErrorKind::UnresolvableType(name) => write!(f, "Unable to resolve type: '{}'", name),

            TypecheckerErrorKind::UnsupportedStatement(kind) => {
                write!(f, "Unable to type-check statement: '{:?}'", kind)
            }

            TypecheckerErrorKind::UnsupportedExpression(kind) => {
                write!(f, "Unable to type-check expression: '{:?}'", kind)
            }

            TypecheckerErrorKind::MissingContext => write!(
                f,
                "An internal error occurred in the typechecker: unable to find a context"
            ),

            TypecheckerErrorKind::ExpectedType { expected, received } => {
                write!(f, "Expected type '{}' but received type '{}'", expected, received)
            }

            TypecheckerErrorKind::MissingReturnStatement => write!(
                f,
                "A return statement was not found in the function block, and the function's return type is not void"
            ),

            TypecheckerErrorKind::DuplicateVariableDeclaration => {
                write!(f, "A variable with the same name already exists in this scope")
            }

            TypecheckerErrorKind::UndeclaredVariable(name) => write!(f, "Undeclared variable: '{}'", name),
        }
    }
}

impl ErrorKind for TypecheckerErrorKind {}
