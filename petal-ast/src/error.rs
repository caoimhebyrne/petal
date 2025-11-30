use enum_display::EnumDisplay;
use petal_core::{
    error::{Error, ErrorKind},
    source_span::SourceSpan,
};
use petal_lexer::token::{Token, TokenKind};

#[derive(Debug, PartialEq, EnumDisplay)]
pub enum ASTError {
    /// A token was encountered in the stream that was unexpected.
    #[display("Unexpected token: {0:?}")]
    UnexpectedToken(TokenKind),

    /// A token was encountered in the stream that was not a certain token.
    #[display("Expected token: {received:?}, but received a different token: {expected:?}")]
    ExpectedToken { received: TokenKind, expected: TokenKind },

    /// A token was encountered in the stream that was not an identifier.
    #[display("Expected an identifier, but received a different token: {0:?}")]
    ExpectedIdentifier(TokenKind),

    /// The end of the token stream was encountered.
    UnexpectedEndOfFile,
}

impl ASTError {
    /// Creates a new [Error] with the kind as an [ASTError::UnexpectedToken] kind.
    pub fn unexpected_token(token: Token) -> Error {
        Error::new(ASTError::UnexpectedToken(token.kind), token.span)
    }

    /// Creates a new [Error] with the kind as an [ASTError::ExpectedToken] kind.
    pub fn expected_token(expected: TokenKind, received: Token) -> Error {
        Error::new(
            ASTError::ExpectedToken {
                received: received.kind,
                expected,
            },
            received.span,
        )
    }

    /// Creates a new [Error] with the kind as an [ASTError::ExpectedIdentifier] kind.
    pub fn expected_identifier(token: Token) -> Error {
        Error::new(ASTError::ExpectedIdentifier(token.kind), token.span)
    }

    /// Creates a new [Error] with the kind as an [ASTError::UnexpectedEndOfFile] kind.
    pub fn unexpected_end_of_file() -> Error {
        Error::new(ASTError::UnexpectedEndOfFile, SourceSpan { start: 0, end: 0 })
    }
}

impl ErrorKind for ASTError {}
