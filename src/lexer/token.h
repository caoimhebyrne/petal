#ifndef __LEXER_TOKEN_H__
#define __LEXER_TOKEN_H__

#include "core/position.h"
#include "util/vector.h"

// The type of a token produced by the Lexer.
typedef enum {
    // An invalid token.
    TOKEN_TYPE_INVALID = 0,

    // An identifier token, e.g. "my_variable"
    TOKEN_TYPE_IDENTIFIER,

    // A keyword token, e.g. "func"
    TOKEN_TYPE_KEYWORD,

    TOKEN_TYPE_EQUALS,    // =
    TOKEN_TYPE_SEMICOLON, // ;
} TokenType;

// A single token produced by the Lexer.
typedef struct {
    // The type of this token.
    TokenType type;

    // The position that this token occurred at within the source file.
    Position position;

    union {
        // Only available on TOKEN_TYPE_IDENTIFIER and TOKEN_TYPE_KEYWORD tokens.
        char* string;
    };
} Token;

#define TOKEN_INVALID (Token){0}

// Destroys a single token.
// This will de-allocate any data stored within the token, e.g. `string`.
// Parameters:
// - token: The token to destroy.
void token_destroy(Token token);

typedef Vector(Token) TokenVector;

#endif // __LEXER_TOKEN_H__
