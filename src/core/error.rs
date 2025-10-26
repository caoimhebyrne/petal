use std::fmt::{Debug, Display};

use crate::core::{dyn_compare::DynCompare, source_span::SourceSpan};

/// A trait for all error kind enums to implement.
pub trait ErrorKind: Display + Debug + DynCompare {}

impl PartialEq<dyn ErrorKind> for dyn ErrorKind {
    fn eq(&self, other: &dyn ErrorKind) -> bool {
        self.dyn_eq(other)
    }
}

#[derive(Debug)]
pub struct Error {
    /// The kind of error that this is.
    pub kind: Box<dyn ErrorKind>,

    /// The location in the source that the error occurred at.
    pub span: SourceSpan,
}

/// A result type where [E] is [Error].
pub type Result<T> = core::result::Result<T, Error>;

impl Error {
    /// Creates a new [Error] with the provided [ErrorKind].
    pub fn new<K: ErrorKind + 'static>(kind: K, span: SourceSpan) -> Self {
        Error {
            kind: Box::new(kind),
            span,
        }
    }
}

impl PartialEq for Error {
    fn eq(&self, other: &Self) -> bool {
        self.kind.as_ref() == other.kind.as_ref() && self.span == other.span
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.kind)
    }
}

/// Allows .into() to be called on an `Error to convert it into a `Result<T, Error>`.
impl<T> From<Error> for core::result::Result<T, Error> {
    fn from(value: Error) -> Self {
        return Err(value);
    }
}
