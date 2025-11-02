use petal_core::error::Result;
use petal_lexer::{stream::TokenStream, token::Token};

use crate::error::ASTErrorKind;

/// An "extension trait" which adds result-variants of [TokenStream] functions.
pub(crate) trait TokenStreamExt {
    /// Returns the next token in the stream that is not considered to be whitespace.
    fn next_non_whitespace_or_err(&mut self) -> Result<&Token>;

    /// Returns the next token in the stream that is not a comment token, without advancing the iterator.
    fn peek_non_whitespace_or_err(&self) -> Result<&Token>;
}

impl TokenStreamExt for TokenStream {
    fn next_non_whitespace_or_err(&mut self) -> Result<&Token> {
        self.next_non_whitespace()
            .ok_or_else(|| ASTErrorKind::unexpected_end_of_file())
    }

    fn peek_non_whitespace_or_err(&self) -> Result<&Token> {
        self.peek_non_whitespace()
            .ok_or_else(|| ASTErrorKind::unexpected_end_of_file())
    }
}
