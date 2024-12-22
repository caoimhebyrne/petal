#ifndef __TOKEN_H__
#define __TOKEN_H__

#include "../position.h"
#include "../stream.h"

// Represents the type of a single Token.
typedef enum {
    // An invalid token.
    TOKEN_INVALID = 0,

    // An identifier, e.g. "my_variable".
    TOKEN_IDENTIFIER = 1,

    // A number literal, e.g. 123.456.
    TOKEN_NUMBER_LITERAL = 2,

    // Symbols
    TOKEN_EQUALS = 3,            // =
    TOKEN_SEMICOLON = 4,         // ;
    TOKEN_SLASH = 5,             // /
    TOKEN_OPEN_PARENTHESIS = 6,  // (
    TOKEN_CLOSE_PARENTHESIS = 7, // )
    TOKEN_OPEN_BRACE = 8,        // {
    TOKEN_CLOSE_BRACE = 9,       // }
    TOKEN_ASTERISK = 10,         // *
} TokenType;

// Represents a single Token produced by the Lexer.
typedef struct {
    // The type of this token.
    // This indicates which value properties are available for access.
    TokenType type;

    // The position in the file that this token occurred at.
    Position position;

    union {
        // Only available on TOKEN_IDENTIFIER and TOKEN_KEYWORD.
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
char* token_to_string(Token* token);

// Returns a string representation of the provided TokenType.
// Parameters:
// - token_type: The token type to turn into a string.
// Returns:
// - A string representing the provided token type.
char* token_type_to_string(TokenType token_type);

DECLARE_STREAM(TokenStream, token_stream, Token);

#endif // __TOKEN_H__
