#pragma once

#include "array.h"
#include "module.h"
#include <stdbool.h>

/**
 * A single kind of token.
 */
typedef enum {
    TOKEN_KIND_IDENTIFIER,
    TOKEN_KIND_NUMBER,
    TOKEN_KIND_EQUALS,
    TOKEN_KIND_OPEN_PARENTHESIS,
    TOKEN_KIND_CLOSE_PARENTHESIS,
    TOKEN_KIND_OPEN_BRACE,
    TOKEN_KIND_CLOSE_BRACE,
    TOKEN_KIND_COLON,
    TOKEN_KIND_SEMICOLON,
    TOKEN_KIND_COMMA,
    TOKEN_KIND_HYPHEN,
    TOKEN_KIND_RIGHT_ANGLE_BRACKET,
    TOKEN_KIND_SLASH,
} TokenKind;

/**
 * A position that a token occurred at within a source file.
 */
typedef struct {
    // The module that the token was in.
    ModuleId module_id;

    // The line that the token started at.
    size_t line;

    // The column that the token started at.
    size_t column;

    // The length of the token.
    size_t length;
} Position;

/**
 * A token that occurred within a source file.
 */
typedef struct {
    // The kind of token that this is.
    TokenKind kind;

    union {
        // Only available on TOKEN_KIND_IDENTIFIER.
        const char* string;

        // Only available on TOKEN_KIND_NUMBER.
        float number;
    };
} Token;

/**
 * An array of tokens.
 */
DEFINE_ARRAY_TYPE(TokenArray, token_array, Token)

/**
 * A lexer used to parse source code into an array of tokens.
 */
typedef struct {
    // The allocator to use when allocating memory.
    Allocator* allocator;

    // The ID of the module that the source code belongs to.
    ModuleId module_id;

    // The buffer containing the source code to parse.
    const StringBuffer* buffer;

    // The index that the lexer is currently at in the source code.
    size_t cursor;
} Lexer;

/**
 * Initializes a lexer using the data in the provided [Module].
 */
void lexer_init(Lexer* lexer, const Module* module);

/**
 * Attempts to parse all of the tokens available to the lexer, returning false if an error occurs.
 */
bool lexer_parse(Lexer* lexer, TokenArray* tokens);
