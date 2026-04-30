use std::fmt::Display;

use crate::{
    core::{
        error::Error,
        span::Span,
    },
    module_registry::ModuleId,
    typechecker::r#type::Type,
};

/// An error emitted by the C backend.
#[derive(Debug, PartialEq)]
pub struct CBackendError {
    /// The kind of lexer error that this is.
    pub kind: CBackendErrorKind,

    /// The [`Span`] that the error occurred at.
    pub span: Span,
}

/// The different kinds of [`CBackendError`]s that exist.
#[derive(Debug, PartialEq)]
pub enum CBackendErrorKind {
    UnsupportedType(Type),
    UnknownType,
    CompilerInvocationFailed(String),
}

impl CBackendErrorKind {
    /// Creates a [CBackendError] from a [CBackendErrorKind].
    pub fn at(self, span: Span) -> CBackendError {
        CBackendError::new(self, span)
    }

    /// Creates a [CBackendError] from a [CBackendErrorKind].
    pub fn without_span(self, module_id: ModuleId) -> CBackendError {
        CBackendError::new(self, Span::new(module_id, 0, 0))
    }
}

impl CBackendError {
    /// Creates a new [`CBackendError`].
    pub fn new(kind: CBackendErrorKind, span: Span) -> Self {
        Self { kind, span }
    }
}

impl Error for CBackendError {
    fn span(&self) -> Option<Span> {
        Some(self.span)
    }
}

impl Display for CBackendErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CBackendErrorKind::UnsupportedType(r#type) => write!(f, "Unsupported type: '{:?}'", r#type),
            CBackendErrorKind::UnknownType => write!(f, "Unresolved/unknown type"),

            CBackendErrorKind::CompilerInvocationFailed(message) => {
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
