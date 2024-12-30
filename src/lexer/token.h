#ifndef __TOKEN_H__
#define __TOKEN_H__

#include "../position.h"
#include "../stream.h"

// Represents the type of a single Token.
typedef enum {
    // An invalid token.
    TOKEN_INVALID,

    // An identifier, e.g. "my_variable".
    TOKEN_IDENTIFIER,

    // A keyword, e.g. "func".
    TOKEN_KEYWORD,

    // A number literal, e.g. 123.456.
    TOKEN_NUMBER_LITERAL,

    // A string literal, e.g: 'Hello World'.
    TOKEN_STRING_LITERAL,

    // Symbols
    TOKEN_EQUALS,              // =
    TOKEN_SEMICOLON,           // ;
    TOKEN_SLASH,               // /
    TOKEN_OPEN_PARENTHESIS,    // (
    TOKEN_CLOSE_PARENTHESIS,   // )
    TOKEN_OPEN_BRACE,          // {
    TOKEN_CLOSE_BRACE,         // }
    TOKEN_ASTERISK,            // *
    TOKEN_HYPHEN,              // -
    TOKEN_RIGHT_ANGLE_BRACKET, // >
    TOKEN_COLON,               // ;
    TOKEN_COMMA,               // ,
    TOKEN_PLUS,                // +
    TOKEN_AMPERSAND,           // &
    TOKEN_QUESTION_MARK,       // ?
} TokenType;

// Represents a single Token produced by the Lexer.
typedef struct {
    // The type of this token.
    // This indicates which value properties are available for access.
    TokenType type;

    // The position in the file that this token occurred at.
    Position position;

    union {
        // Only available on TOKEN_IDENTIFIER, TOKEN_KEYWORD, and TOKEN_STRING_LITERAL.
        char* string;

        // Only available on TOKEN_NUMBER_LITERAL.
        double number;
    };
} Token;

// An invalid token.
#define INVALID_TOKEN (Token){.type = TOKEN_INVALID, .position = (Position){}};

// Returns a string representation of the provided Token.
// Parameters:
// - token: The token to turn into a string.
// Returns:
// - A string representing the provided token.
char* token_to_string(Token token);

// Returns a string representation of the provided TokenType.
// Parameters:
// - token_type: The token type to turn into a string.
// Returns:
// - A string representing the provided token type.
char* token_type_to_string(TokenType token_type);

DECLARE_STREAM(TokenStream, token_stream, Token);

#endif // __TOKEN_H__
