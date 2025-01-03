#pragma once

#include "core/position.h"
#include "util/vector.h"
#include <stdint.h>

// The type of a token produced by the Lexer.
typedef enum {
    // An invalid token.
    TOKEN_TYPE_INVALID = 0,

    // An identifier token, e.g. "my_variable"
    TOKEN_TYPE_IDENTIFIER,

    // A keyword token, e.g. "func"
    TOKEN_TYPE_KEYWORD,

    // An integer literal, e.g. 123456
    TOKEN_TYPE_INTEGER_LITERAL,

    // A float literal, e.g. 123.456
    TOKEN_TYPE_FLOAT_LITERAL,

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

        // Only available on TOKEN_TYPE_INTEGER_LITERAL tokens.
        uint64_t integer;

        // Only available on TOKEN_TYPE_FLOAT_LITERAL tokens.
        double number;
    };
} Token;

#define TOKEN_INVALID (Token){0}

// Destroys a single token.
// This will de-allocate any data stored within the token, e.g. `string`.
// Parameters:
// - token: The token to destroy.
void token_destroy(Token token);

typedef Vector(Token) TokenVector;
