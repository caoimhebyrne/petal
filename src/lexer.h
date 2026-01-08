#pragma once

#include "array.h"
#include <stdbool.h>

/**
 * A single kind of token.
 */
typedef enum {
    TOKEN_KIND_IDENTIFIER,
} TokenKind;

/**
 * A token that occurred within a source file.
 */
typedef struct {
    // The kind of token that this is.
    TokenKind kind;
} Token;

/**
 * An array of tokens.
 */
DEFINE_ARRAY_TYPE(TokenArray, token_array, Token)

/**
 * A lexer used to parse source code into an array of tokens.
 */
typedef struct {
    // The buffer containing the source code to parse.
    const StringBuffer *buffer;

    // The index that the lexer is currently at in the source code.
    size_t cursor;
} Lexer;

/**
 * Initializes a lexer with the provided [StringBuffer].
 */
void lexer_init(Lexer *lexer, const StringBuffer *buffer);

/**
 * Attempts to parse all of the tokens available to the lexer, returning false if an error occurs.
 */
bool lexer_parse(Lexer *lexer, TokenArray *tokens);
