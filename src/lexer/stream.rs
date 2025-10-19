use crate::lexer::token::Token;

/// A [TokenStream] wraps a [Vec] of [Tokens], allowing the caller to peek and consume at the same time.
pub struct TokenStream {
    /// The [Vec] containing the [Token]s that this stream provides.
    tokens: Vec<Token>,

    /// The current index that the stream is at.
    index: usize,
}

impl TokenStream {
    /// Returns a new [TokenStream] from a [Vec] of [Token]s.
    pub fn new(tokens: Vec<Token>) -> Self {
        TokenStream { tokens, index: 0 }
    }

    /// Returns the token at the current index advancing the stream.
    pub fn next(&mut self) -> Option<&Token> {
        return self.tokens.get(self.index).map(|it| {
            self.index += 1;
            it
        });
    }

    /// Returns the next token in the stream that is not considered to be whitespace.
    pub fn next_non_whitespace(&mut self) -> Option<&Token> {
        while let Some(token) = self.tokens.get(self.index) {
            self.index += 1;

            // If this token is not whitespace, then we can return it.
            if !token.is_considered_whitespace() {
                return Some(token);
            }
        }

        None
    }

    /// Returns the token at the current index without advancing the stream.
    pub fn peek(&self) -> Option<&Token> {
        return self.tokens.get(self.index);
    }

    /// Returns the next token in the stream that is not a comment token, without advancing the iterator.
    pub fn peek_non_whitespace(&self) -> Option<&Token> {
        let mut cursor = self.index;

        while let Some(token) = self.tokens.get(cursor) {
            cursor += 1;

            // If this token is not whitespace, then we can return it.
            if !token.is_considered_whitespace() {
                return Some(token);
            }
        }

        None
    }

    /// Returns whether the end of the stream has been reached.
    pub fn has_remaining(&self) -> bool {
        self.index < self.tokens.len()
    }
}
