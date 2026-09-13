use petal_span::Span;

#[derive(Debug, Clone)]
pub struct Diagnostic {
    /// The message of the [`Diagnostic`].
    pub message: String,

    /// The severity of the [`Diagnostic`]. This indicates whether it shoudl prevent compilation from continuing.
    pub severity: DiagnosticSeverity,

    /// The [`Span`] that the [`Diagnostic`] is targetting.
    pub span: Span,
}

impl Diagnostic {
    /// Create a new error diagnostic at the provided [`Span`] with a message.
    pub fn error(span: Span, message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            severity: DiagnosticSeverity::Error,
            span,
        }
    }

    /// Create a new warning diagnostic at the provided [`Span`] with a message.
    pub fn warning(span: Span, message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            severity: DiagnosticSeverity::Warning,
            span,
        }
    }
}

/// The severity of a [`Diagnostic`].
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum DiagnosticSeverity {
    /// The diagnostic should prevent compilation from continuing.
    Error,

    /// The diagnostic does not prevent compilation from continuing.
    Warning,
}
