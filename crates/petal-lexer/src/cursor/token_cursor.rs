use std::iter::Peekable;

use crate::{Cursor, Token, TokenKind};

/// A wrapper around a peekable iterator of [`Token`]s that implements [`Cursor`].
pub struct TokenCursor<'a> {
    /// The iterator to consume tokens from.
    tokens: Peekable<Box<dyn Iterator<Item = Token> + 'a>>,

    /// Whether the cursor should skip over comments when [`consume`] and [`peek`] are called.
    skip_comments: bool,
}

impl<'a> TokenCursor<'a> {
    /// Create a new [`TokenCursor`] from an [`Iterator`] of [`Token`]s.
    ///
    /// If `skip_comments` is `true`, the cursor will advance past any comment tokens before attempting to [`peek`]
    /// or [`consume`].
    pub fn new(tokens: impl Iterator<Item = Token> + 'a, skip_comments: bool) -> Self {
        // Not exactly sure why the type needs to be so explicit here, but the lifetimes get all muddled up without it.
        let boxed: Box<dyn Iterator<Item = Token>> = Box::new(tokens);

        Self {
            tokens: boxed.peekable(),
            skip_comments,
        }
    }
}

impl Cursor<Token> for TokenCursor<'_> {
    fn consume(&mut self) -> Option<Token> {
        self.skip_comments_if_enabled();
        self.tokens.next()
    }

    fn peek(&mut self) -> Option<Token> {
        self.skip_comments_if_enabled();
        self.tokens.peek().copied()
    }
}

impl TokenCursor<'_> {
    /// Advance the cursor pass any [`TokenKind::Comment`]s.
    ///
    /// `consume_while` calls `consume`, so we must practically re-implement it to skip over comments inside [`consume`]
    /// and [`peek`].
    fn skip_comments_if_enabled(&mut self) {
        if !self.skip_comments {
            return;
        }

        while self.tokens.peek().map_or_default(|it| it.kind == TokenKind::Comment) {
            self.tokens.next();
        }
    }
}
