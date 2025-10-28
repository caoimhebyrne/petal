use std::collections::HashMap;

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
}

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
