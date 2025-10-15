use std::fmt::Display;

/// Represents the different kinds of errors that can be returned when parsing an AST.
#[derive(Debug, Clone, PartialEq)]
pub enum ASTErrorKind {
    /// The end of the file was reached when it was not expected.
    UnexpectedEndOfFile,
}

impl Display for ASTErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ASTErrorKind::UnexpectedEndOfFile => write!(f, "Unexpected end-of-file"),
        }
    }
}
