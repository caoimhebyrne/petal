/// A range within some text, i.e. a start and end offset in bytes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    /// The zero-indexed start offset of the span.
    start: u32,

    /// The zero-indexed end offset of the span.
    end: u32,
}

impl Span {
    /// Create a new [`Span`] from a `start` and `end` offset.
    ///
    /// # Panics
    ///
    /// This function will panic if `end` is before `start`.
    pub const fn new(start: u32, end: u32) -> Self {
        assert!(end > start, "The end of a span must be after the start");
        Self { start, end }
    }

    /// Get the number of bytes that this [`Span`] covers.
    pub const fn size(self) -> u32 {
        self.end - self.start
    }

    /// Returns the slice of the provided `string` that this [`Span`] covers.
    ///
    /// # Panics
    ///
    /// This function will panic if this [`Span`] is outside the bounds of the provided `string`.
    pub fn slice(self, string: &str) -> &str {
        &string[self.start as usize..self.end as usize]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let span = Span::new(10, 14);
        assert_eq!(span.start, 10);
        assert_eq!(span.end, 14);
    }

    #[test]
    fn size_returns_correct_value() {
        let span = Span::new(0, 4);
        assert_eq!(span.size(), 4);
    }

    #[test]
    fn slice_returns_correct_value() {
        let source = "func main() {}";
        let span = Span::new(5, 9);
        assert_eq!(span.slice(source), "main");
    }

    #[test]
    #[should_panic(expected = "The end of a span must be after the start")]
    fn new_throws_with_invalid_end_offset() {
        let _ = Span::new(2, 0);
    }
}
