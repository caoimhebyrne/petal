/// A range within some text, i.e. a start and end offset in bytes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
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

    /// Return a new [`Span`] starting at `self` and ending at `end`.
    ///
    /// # Panics
    ///
    /// This function will panic if `end` is before `self`.
    #[must_use]
    pub const fn until(self, end: Self) -> Self {
        // TODO: I'm not sure if the bounds checking is correct here.
        assert!(
            end.start > self.start,
            "The `end` span must start after the end of `self`"
        );

        Self::new(self.start, end.end)
    }

    /// Get the start index of this [`Span`].
    pub const fn start(self) -> u32 {
        self.start
    }

    /// Get the end index of this [`Span`].
    pub const fn end(self) -> u32 {
        self.end
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
    fn until_returns_correct_value() {
        let a = Span::new(0, 4);
        let b = Span::new(2, 6);
        assert_eq!(a.until(b), Span::new(0, 6));
    }

    #[test]
    #[should_panic(expected = "The end of a span must be after the start")]
    fn new_throws_with_invalid_end_offset() {
        let _ = Span::new(2, 0);
    }
}
