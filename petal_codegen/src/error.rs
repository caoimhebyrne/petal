use std::fmt::Display;

use petal_core::core::location::Location;

#[derive(Debug, Copy, Clone)]
pub enum DriverErrorKind {}

#[derive(Debug, Copy, Clone)]
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
}

impl Display for DriverError {
    fn fmt(&self, _f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.kind {}
    }
}
