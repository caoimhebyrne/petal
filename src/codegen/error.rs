use crate::core::location::Location;
use std::fmt::Display;

#[derive(Debug, Clone)]
pub enum CodegenErrorKind {
    InternalError(String),
    VerificationError(String),
}

#[derive(Debug, Clone)]
pub struct CodegenError {
    pub kind: CodegenErrorKind,
    pub location: Option<Location>,
}

impl<'a> CodegenError {
    pub fn internal_error(message: String, location: Option<Location>) -> CodegenError {
        CodegenError {
            kind: CodegenErrorKind::InternalError(message),
            location,
        }
    }

    pub fn verification_error(message: String, location: Option<Location>) -> CodegenError {
        CodegenError {
            kind: CodegenErrorKind::VerificationError(message),
            location,
        }
    }
}

impl Display for CodegenError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.kind {
            CodegenErrorKind::InternalError(message) => write!(f, "Internal code generation error: {}", message),
            CodegenErrorKind::VerificationError(message) => {
                write!(
                    f,
                    "Code generation verification error (this is likely a bug!): {}",
                    message
                )
            }
        }
    }
}
