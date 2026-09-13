use std::str::Chars;

use crate::cursor::Cursor;

/// A wrapper around a string slice and a [`Chars`] iterator, which allows you to peek ahead past the next character
/// in the iterator.
pub struct CharCursor<'a> {
    /// The string slice that this cursor is wrapping.
    string: &'a str,

    /// The iterator over the characters of `source`.
    chars: Chars<'a>,
}

impl<'a> CharCursor<'a> {
    /// Create a new [`Cursor`] from a string slice.
    pub fn new(string: &'a str) -> Self {
        Self {
            string,
            chars: string.chars(),
        }
    }

    /// Return the offset that the cursor is currently at compared to the start of the source string.
    pub fn offset(&self) -> usize {
        // `as_str` returns a pointer to a string slice, which is cheap, and `len` just peeks into that slice's
        // metadata, so this operation is not expensive.
        self.string.len() - self.chars.as_str().len()
    }
}

impl Cursor<char> for CharCursor<'_> {
    fn consume(&mut self) -> Option<char> {
        self.chars.next()
    }

    fn peek(&mut self) -> Option<char> {
        // cloning the iterator is cheap, as it only clones the pointer that the iterator is currently at, alongside
        // some metadata.
        self.chars.clone().next()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn consume_returns_next_character() {
        let string = "ab";
        let mut cursor = CharCursor::new(string);

        assert_eq!(cursor.consume(), Some('a'));
        assert_eq!(cursor.consume(), Some('b'));
        assert_eq!(cursor.consume(), None);
    }

    #[test]
    fn consume_while_does_not_consume_predicate_character() {
        let string = "Hello, world!";
        let mut cursor = CharCursor::new(string);

        cursor.consume_while(|char| char != ',');

        assert_eq!(cursor.consume(), Some(','));
    }

    #[test]
    fn consume_if_does_not_consume_when_does_not_match() {
        let string = "hello";
        let mut cursor = CharCursor::new(string);

        assert!(cursor.consume_if(|it| it == 'x').is_none());
        assert_eq!(cursor.consume(), Some('h'));
    }

    #[test]
    fn offset_returns_correct_value_after_consume() {
        let string = "Hello, world!";
        let mut cursor = CharCursor::new(string);

        let _ = cursor.consume();
        assert_eq!(cursor.offset(), 1);
    }

    #[test]
    fn peek_returns_next_character_without_consuming() {
        let string = "Hello, world!";
        let mut cursor = CharCursor::new(string);

        assert_eq!(cursor.peek(), Some('H'));
        assert_eq!(cursor.offset(), 0);
    }
}
