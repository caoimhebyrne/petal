use std::{collections::HashMap, fmt::Display};

use crate::{
    error::{Error, ErrorKind, Result},
    source_span::SourceSpan,
};

/// A reference to an intern'd string.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct StringReference(pub usize);

/// A pool of 'intern'ed strings.
/// This ensures that we only have one distinct `String` value allocated during the lifetime of the compiler.
/// This is used, for example, when creating an `Identifier` token.
pub trait StringInternPool {
    /// Attempts to intern the provided string reference. If the same string has already been allocated, then a
    /// reference to the already-allocated [String] will be returned.
    fn intern(&mut self, value: &str) -> StringReference;

    /// Returns a string slice for the provided [StringReference], if it exists.
    fn resolve_reference(&self, reference: &StringReference) -> Option<&str>;

    /// Returns a string slice for the provided [StringReference].
    ///
    /// Errors:
    /// - [StringInternPoolError::UndefinedStringReference] If a string could not be found for the provided reference.
    fn resolve_reference_or_err(&self, reference: &StringReference, span: SourceSpan) -> Result<&str>;
}

/// A "default" implementation of [StringInternPool] which copies strings to a [Vec] when interning them.
pub struct StringInternPoolImpl {
    /// A map of `String` values to their `StringReference`.
    string_index_lookup_map: HashMap<String, StringReference>,

    /// A [Vec] of allocated `String` values.
    allocated_strings: Vec<String>,
}

impl StringInternPoolImpl {
    /// Creates a new [StringInternPool] instance.
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        StringInternPoolImpl {
            string_index_lookup_map: HashMap::new(),
            allocated_strings: Vec::new(),
        }
    }
}

impl StringInternPool for StringInternPoolImpl {
    fn intern(&mut self, value: &str) -> StringReference {
        // If a value already exists in the map, then we can return its reference.
        if let Some(reference) = self.string_index_lookup_map.get(value) {
            return *reference;
        }

        // Otherwise, we can allocate a new string.
        let reference = StringReference(self.string_index_lookup_map.len());

        // FIXME: This implementation is not *ideal*, we're making two copies of the string. We could instead reference
        //        a span in the original source string, and return that when resolving the reference.
        self.string_index_lookup_map.insert(value.into(), reference);
        self.allocated_strings.push(value.into());

        reference
    }

    fn resolve_reference(&self, reference: &StringReference) -> Option<&str> {
        self.allocated_strings.get(reference.0).map(|it| it.as_str())
    }

    fn resolve_reference_or_err(&self, reference: &StringReference, span: SourceSpan) -> Result<&str> {
        self.resolve_reference(reference)
            .ok_or(StringInternPoolError::undefined_string_reference(*reference, span))
    }
}

#[derive(Debug, PartialEq)]
pub enum StringInternPoolError {
    UndefinedStringReference(StringReference),
}

impl StringInternPoolError {
    /// Creates a new [Error] with the kind as a [StringInternPoolError::UndefinedStringReference] kind.
    pub fn undefined_string_reference(reference: StringReference, span: SourceSpan) -> Error {
        Error::new(StringInternPoolError::UndefinedStringReference(reference), span)
    }
}

impl Display for StringInternPoolError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StringInternPoolError::UndefinedStringReference(reference) => {
                write!(f, "Undefined string reference: '{:?}'", reference)
            }
        }
    }
}

impl ErrorKind for StringInternPoolError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_intern_one_string() {
        let mut pool = StringInternPoolImpl::new();
        let reference = pool.intern("Hello, world!");
        let string = pool.resolve_reference(&reference);

        assert_eq!(string, Some("Hello, world!"));
    }

    #[test]
    fn can_intern_many_strings() {
        let mut pool = StringInternPoolImpl::new();

        for i in 0..10 {
            let expected_string = format!("String {}", i + 1);
            let reference = pool.intern(&expected_string);
            assert_eq!(reference, StringReference(i));

            let string = pool.resolve_reference(&reference);
            assert_eq!(string, Some(expected_string.as_str()));
        }
    }

    #[test]
    fn does_not_allocate_more_than_once() {
        let mut pool = StringInternPoolImpl::new();

        for _ in 0..10 {
            let reference = pool.intern("Hello, world!");
            assert_eq!(reference, StringReference(0));
        }
    }
}
