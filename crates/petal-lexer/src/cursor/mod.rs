mod char_cursor;
mod token_cursor;

pub use char_cursor::*;
pub use token_cursor::*;

/// An iterator-like trait.
///
/// Provides `consume()`, which returns the next element in the iterator, and `peek()` which returns an optional
/// reference to the next element.
pub trait Cursor<Item: Copy> {
    /// Return the next item in the iterator, advancing the cursor.
    fn consume(&mut self) -> Option<Item>;

    /// Return at the next item in the iterator, without advancing the cursor.
    ///
    /// This function takes a mutable reference to self to make it compatible with the `Peekable::peek` API.
    fn peek(&mut self) -> Option<Item>;

    /// Return true and advance the cursor if `predicate` is true.
    fn consume_if(&mut self, predicate: impl FnOnce(Item) -> bool) -> Option<Item> {
        self.peek().map_or_default(predicate).then(|| self.consume()).flatten()
    }

    /// Advance the cursor until the `predicate` returns false, or until there are no more items left to consume.
    fn consume_while(&mut self, predicate: impl Fn(Item) -> bool) {
        while self.consume_if(&predicate).is_some() {}
    }
}
