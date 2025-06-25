use std::fmt::Display;

use petal_core::core::location::Location;

#[derive(Debug, Clone)]
pub enum DriverErrorKind {
    CompilationFailure,
    UnableToWrite { file_name: String, message: String },
}

#[derive(Debug, Clone)]
pub struct DriverError {
    /// The kind of error that occurred.
    pub kind: DriverErrorKind,

    /// The position that the error occurred at within the source file.
    pub location: Option<Location>,
}

pub type DriverResult<T> = Result<T, DriverError>;

impl DriverError {
    pub fn new(kind: DriverErrorKind, location: Option<Location>) -> DriverError {
        DriverError { kind, location }
    }

    pub fn compilation_failure(location: Option<Location>) -> DriverError {
        DriverError::new(DriverErrorKind::CompilationFailure, location)
    }

    pub fn unable_to_write(file_name: String, message: String, location: Option<Location>) -> DriverError {
        DriverError::new(DriverErrorKind::UnableToWrite { file_name, message }, location)
    }
}

impl Display for DriverError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.kind {
            DriverErrorKind::CompilationFailure => {
                write!(f, "Failed to compile, see the logs above for more information.")
            }

            DriverErrorKind::UnableToWrite { file_name, message } => {
                write!(f, "Failed to write to file '{}': {}", file_name, message)
            }
        }
    }
}
