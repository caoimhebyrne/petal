use std::fmt::Display;

use crate::{
    core::{
        error::Error,
        span::Span,
    },
    typechecker::{
        context::{
            FunctionId,
            StructureId,
        },
        r#type::Type,
    },
};

/// An error emitted by the C backend.
#[derive(Debug, PartialEq)]
pub struct CBackendError {
    /// The kind of lexer error that this is.
    pub kind: CBackendErrorKind,

    /// The [`Span`] that the error occurred at.
    pub span: Option<Span>,
}

/// The different kinds of [`CBackendError`]s that exist.
#[derive(Debug, PartialEq)]
pub enum CBackendErrorKind {
    MissingStructureId,
    MissingStructure(StructureId),
    MissingFunctionId,
    MissingFunction(FunctionId),
    UnsupportedType(Type),
    UnknownType,
    CompilerInvocationFailed(String),
}

impl CBackendErrorKind {
    /// Creates a [CBackendError] from a [CBackendErrorKind].
    pub fn at(self, span: Span) -> CBackendError {
        CBackendError::new(self, Some(span))
    }

    /// Creates a [CBackendError] from a [CBackendErrorKind].
    pub fn without_span(self) -> CBackendError {
        CBackendError::new(self, None)
    }
}

impl CBackendError {
    /// Creates a new [`CBackendError`].
    pub fn new(kind: CBackendErrorKind, span: Option<Span>) -> Self {
        Self { kind, span }
    }
}

impl Error for CBackendError {
    fn span(&self) -> Option<Span> {
        self.span
    }
}

impl Display for CBackendErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingStructureId => write!(f, "Expression was never patched to include a structure ID!"),

            Self::MissingFunctionId => write!(f, "Expression was never patched to include a function ID!"),

            Self::MissingStructure(id) => {
                write!(f, "Structure with ID '{id}' was referenced, but did not find a matching structure definition")
            }

            Self::MissingFunction(id) => {
                write!(f, "Function with ID '{id}' was referenced, but could not find a matching definition")
            }

            Self::UnsupportedType(r#type) => write!(f, "Unsupported type: '{:?}'", r#type),

            Self::UnknownType => write!(f, "Unresolved/unknown type"),

            Self::CompilerInvocationFailed(message) => {
                write!(f, "Failed to invoke C compiler: '{message}'")
            }
        }
    }
}

impl Display for CBackendError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.kind.fmt(f)
    }
}
