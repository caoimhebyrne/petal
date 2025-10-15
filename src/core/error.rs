use crate::core::source_span::SourceSpan;
use std::fmt::{Debug, Display};

#[derive(Debug, PartialEq, PartialOrd)]
pub struct Error<K: Debug + Display> {
    /// The kind of error that this is.
    pub kind: K,

    /// The location in the source that the error occurred at.
    pub span: SourceSpan,
}

/// Allows .into() to be called on an `Error<K>` to convert it into a `Result<T, Error<K>>`.
impl<T, K: Debug + Display> From<Error<K>> for Result<T, Error<K>> {
    fn from(value: Error<K>) -> Result<T, Error<K>> {
        return Err(value);
    }
}
