use std::fmt::Display;

use petal_ast::statement::{Statement, StatementKind};
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
        }
    }
}

impl ErrorKind for TypecheckerErrorKind {}
