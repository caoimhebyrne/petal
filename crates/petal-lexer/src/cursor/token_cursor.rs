use std::iter::Peekable;

use crate::{Cursor, Token};

/// A wrapper around a peekable iterator of [`Token`]s that implements [`Cursor`].
pub struct TokenCursor<'a> {
    /// The iterator to consume tokens from.
    tokens: Peekable<Box<dyn Iterator<Item = Token> + 'a>>,
}

impl<'a> TokenCursor<'a> {
    /// Create a new [`TokenCursor`] from an [`Iterator`] of [`Token`]s.
    pub fn new(tokens: impl Iterator<Item = Token> + 'a) -> Self {
        // Not exactly sure why the type needs to be so explicit here, but the lifetimes get all muddled up without it.
        let boxed: Box<dyn Iterator<Item = Token>> = Box::new(tokens);

        Self {
            tokens: boxed.peekable(),
        }
    }
}

impl Cursor<Token> for TokenCursor<'_> {
    fn consume(&mut self) -> Option<Token> {
        self.tokens.next()
    }

    fn peek(&mut self) -> Option<Token> {
        self.tokens.peek().copied()
    }
}
