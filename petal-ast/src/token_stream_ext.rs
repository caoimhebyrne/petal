use crate::error::ASTError;
use petal_core::{error::Result, source_span::SourceSpan, string_intern::StringReference};
use petal_lexer::{
    stream::TokenStream,
    token::{Token, TokenKind},
};

pub trait TokenStreamExt {
    /// Returns the token at the current index advancing the stream, or an [Err] containing an [ASTError::EndOfFile].
    fn consume_or_err(&mut self) -> Result<&Token>;

    /// Returns the token at the current index of the stream, or an [Err] containing an [ASTError::EndOfFile].
    fn peek_or_err(&mut self) -> Result<&Token>;

    /// Expects a certain token to be at the current position of the stream. If it is, it will be consumed.
    fn expect(&mut self, kind: TokenKind) -> Result<&Token>;

    /// Expects an identifier to be at the current position of the stream. If it is, it will be consumed.
    fn expect_identifier(&mut self) -> Result<(StringReference, SourceSpan)>;
}

impl TokenStreamExt for TokenStream {
    fn consume_or_err(&mut self) -> Result<&Token> {
        self.consume_non_whitespace().ok_or(ASTError::unexpected_end_of_file())
    }

    fn peek_or_err(&mut self) -> Result<&Token> {
        self.peek_non_whitespace().ok_or(ASTError::unexpected_end_of_file())
    }

    fn expect(&mut self, kind: TokenKind) -> Result<&Token> {
        let token = self.consume_or_err()?;
        if token.kind != kind {
            return ASTError::unexpected_token(*token).into();
        }

        Ok(token)
    }

    fn expect_identifier(&mut self) -> Result<(StringReference, SourceSpan)> {
        let token = self.consume_or_err()?;

        match token.kind {
            TokenKind::Identifier(reference) => Ok((reference, token.span)),
            _ => return ASTError::expected_identifier(*token).into(),
        }
    }
}
