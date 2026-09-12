use std::str::Chars;

/// A wrapper around a string slice and a [`Chars`] iterator, which allows you to peek ahead past the next character
/// in the iterator.
pub struct Cursor<'a> {
    /// The string slice that this cursor is wrapping.
    string: &'a str,

    /// The iterator over the characters of `source`.
    chars: Chars<'a>,
}

impl<'a> Cursor<'a> {
    /// Create a new [`Cursor`] from a string slice.
    pub fn new(string: &'a str) -> Self {
        Self {
            string,
            chars: string.chars(),
        }
    }

    /// Consumes the next character in the iterator.
    pub fn consume(&mut self) -> Option<char> {
        self.chars.next()
    }

    /// Peeks at the next character in the iterator.
    pub fn peek(&self) -> Option<char> {
        // cloning the iterator is cheap, as it only clones the pointer that the iterator is currently at, alongside
        // some metadata.
        self.chars.clone().next()
    }

    /// Peeks at the character at the provided offset from the iterator's current position.
    pub fn peek_nth(&self, offset: usize) -> Option<char> {
        // cloning the iterator is cheap, as it only clones the pointer that the iterator is currently at, alongside
        // some metadata.
        self.chars.clone().nth(offset)
    }

    /// Consumes characters from the iterator until the `predicate` returns false, or the end of the iterator is
    /// reached.
    pub fn consume_while(&mut self, predicate: impl Fn(char) -> bool) {
        while self.peek().map_or_default(&predicate) {
            let _ = self.consume();
        }
    }

    /// Returns the offset that the iterator is currently at compared to the start of the source string.
    pub fn offset(&self) -> usize {
        // `as_str` returns a pointer to a string slice, which is cheap, and `len` just peeks into that slice's
        // metadata, so this operation is not expensive.
        self.string.len() - self.chars.as_str().len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn consume_returns_next_character() {
        let string = "ab";
        let mut cursor = Cursor::new(string);

        assert_eq!(cursor.consume(), Some('a'));
        assert_eq!(cursor.consume(), Some('b'));
        assert_eq!(cursor.consume(), None);
    }

    #[test]
    fn consume_while_does_not_consume_predicate_character() {
        let string = "Hello, world!";
        let mut cursor = Cursor::new(string);

        cursor.consume_while(|char| char != ',');

        assert_eq!(cursor.consume(), Some(','));
    }

    #[test]
    fn offset_returns_correct_value_after_consume() {
        let string = "Hello, world!";
        let mut cursor = Cursor::new(string);

        let _ = cursor.consume();
        assert_eq!(cursor.offset(), 1);
    }

    #[test]
    fn peek_returns_next_character_without_consuming() {
        let string = "Hello, world!";
        let cursor = Cursor::new(string);

        assert_eq!(cursor.peek(), Some('H'));
        assert_eq!(cursor.offset(), 0);
    }

    #[test]
    fn peek_nth_returns_without_consuming() {
        let string = "Hello, world!";
        let mut cursor = Cursor::new(string);

        assert_eq!(cursor.peek_nth(5), Some(','));
        assert_eq!(cursor.consume(), Some('H'));

        assert_eq!(cursor.peek_nth(3), Some('o'));
        assert_eq!(cursor.consume(), Some('e'));
    }
}
