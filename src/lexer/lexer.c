#include "lexer/lexer.h"
#include "core/diagnostic.h"
#include "core/position.h"
#include "lexer/token.h"
#include "util/defer.h"
#include "util/file.h"
#include "util/format.h"
#include "util/string_builder.h"
#include "util/vector.h"
#include <ctype.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

// A list of identifiers that should be treated as keywords.
char* keywords[] = {"func"};

// Parsing functions:
Token lexer_parse_identifier(Lexer* lexer);
Token lexer_parse_number(Lexer* lexer);

// Forward declarations:
// Whether the lexer has reached the end of the file or not.
bool lexer_is_eof(Lexer* lexer);

// Returns the next character to be parsed, lexer_is_eof should be called before this.
char lexer_peek(Lexer* lexer);

// Consumes a character from the file, advancing the position cursor.
char lexer_consume(Lexer* lexer);

// Produces a single-character token while consuming the current token.
Token lexer_create_token(Lexer* lexer, TokenType token_type);

Lexer lexer_create(DiagnosticVector* diagnostics, FileContents contents) {
    return (Lexer){
        .diagnostics = diagnostics,
        .contents = contents,
        .position = (Position){},
    };
}

TokenVector lexer_parse(Lexer* lexer) {
    TokenVector vector = vector_create();
    if (!vector_initialize(vector, 2)) {
        return (TokenVector){};
    }

    // Keep parsing until the lexer reaches the end of the file.
    while (!lexer_is_eof(lexer)) {
        auto character = lexer_peek(lexer);
        switch (character) {
        // Ignore whitespace (including tabs and CR).
        case ' ':
        case '\t':
        case '\r':
            lexer_consume(lexer);
            break;

        case '\n':
            lexer_consume(lexer);

            // We must also advance the line cursor, and reset the column.
            lexer->position.line++;
            lexer->position.column = 0;

            break;

        case '=':
            vector_append(&vector, lexer_create_token(lexer, TOKEN_TYPE_EQUALS));
            break;

        case ';':
            vector_append(&vector, lexer_create_token(lexer, TOKEN_TYPE_SEMICOLON));
            break;

        case '+':
            vector_append(&vector, lexer_create_token(lexer, TOKEN_TYPE_PLUS));
            break;

        case '-':
            vector_append(&vector, lexer_create_token(lexer, TOKEN_TYPE_MINUS));
            break;

        case '*':
            vector_append(&vector, lexer_create_token(lexer, TOKEN_TYPE_ASTERISK));
            break;

        case '/':
            vector_append(&vector, lexer_create_token(lexer, TOKEN_TYPE_SLASH));
            break;

        default:
            // If the character is an alphabetic character, it is most likely an identifier.
            if (isalpha(character)) {
                auto token = lexer_parse_identifier(lexer);
                if (token.type != TOKEN_TYPE_INVALID) {
                    vector_append(&vector, token);
                    continue;
                }
            } else if (isdigit(character)) {
                auto token = lexer_parse_number(lexer);
                if (token.type != TOKEN_TYPE_INVALID) {
                    vector_append(&vector, token);
                    continue;
                }
            } else {
                vector_append(
                    lexer->diagnostics,
                    diagnostic_create(lexer->position, format_string("unexpected character: '%c'", character))
                );
            }

            // Destroy the original vector, it may have tokens in it already/
            vector_destroy(vector, token_destroy);

            // Return an invalid vector.
            return (TokenVector){};
        }
    }

    return vector;
}

Token lexer_create_token(Lexer* lexer, TokenType token_type) {
    // The current position is the token's position.
    auto position = lexer->position;
    position.length = 1;

    // Advance the cursor.
    lexer_consume(lexer);

    return (Token){
        .type = token_type,
        .position = position,
    };
}

bool lexer_next_is_identifier(Lexer* lexer) {
    auto character = lexer_peek(lexer);
    return isalpha(character) || isdigit(character) || character == '_';
}

Token lexer_parse_identifier(Lexer* lexer) {
    // This token starts at the lexer's current position.
    auto position = lexer->position;

    // An identifier must only contain alphanumeric characters or underscores.
    auto builder = string_builder_create();
    if (string_builder_is_invalid(builder)) {
        vector_append(lexer->diagnostics, DIAGNOSTIC_INTERNAL_ERROR(position));
        return TOKEN_INVALID;
    }

    while (lexer_next_is_identifier(lexer)) {
        string_builder_append(&builder, lexer_consume(lexer));
    }

    // The position's length can be inferred from the length of the string buffer.
    position.length = string_builder_length(builder);

    // Finalizing the builder can fail if it fails to allocate a string of the required length.
    // The builder is no longer safe to use after this.
    char* identifier = string_builder_finish(&builder);
    if (!identifier) {
        vector_append(lexer->diagnostics, DIAGNOSTIC_INTERNAL_ERROR(position));
        return TOKEN_INVALID;
    }

    auto type = TOKEN_TYPE_IDENTIFIER;

    // If the identifier matches a keyword value, treat this token as a keyword.
    for (size_t i = 0; i < sizeof(char*) / sizeof(keywords); i++) {
        if (strcmp(identifier, keywords[i]) == 0) {
            type = TOKEN_TYPE_KEYWORD;
            break;
        }
    }

    return (Token){
        .type = type,
        .position = position,
        .string = identifier,
    };
}

bool lexer_next_is_number(Lexer* lexer) {
    auto character = lexer_peek(lexer);
    return isdigit(character) || character == '.';
}

Token lexer_parse_number(Lexer* lexer) {
    // This token starts at the lexer's current position.
    auto position = lexer->position;

    // An identifier must only contain alphanumeric characters or underscores.
    auto builder = string_builder_create();
    if (string_builder_is_invalid(builder)) {
        vector_append(lexer->diagnostics, DIAGNOSTIC_INTERNAL_ERROR(position));
        return TOKEN_INVALID;
    }

    // Set to true if a `.` is consumed, indicating this is a float.
    auto is_float = false;

    while (lexer_next_is_number(lexer)) {
        auto character = lexer_consume(lexer);
        string_builder_append(&builder, character);

        if (!is_float && character == '.') {
            is_float = true;
        }
    }

    // The position's length can be inferred from the length of the string buffer.
    position.length = string_builder_length(builder);

    // Finalizing the builder can fail if it fails to allocate a string of the required length.
    // The builder is no longer safe to use after this.
    defer(free_str) auto string_value = string_builder_finish(&builder);
    if (!string_value) {
        vector_append(lexer->diagnostics, DIAGNOSTIC_INTERNAL_ERROR(position));
        return TOKEN_INVALID;
    }

    // strtod will set end_ptr to the first character after the numeric value has been parsed.
    auto end_ptr = string_value;
    auto value = strtod(string_value, &end_ptr);

    // If the end_ptr is still equal to the string_value, this is an invalid number.
    if (end_ptr == string_value) {
        vector_append(
            lexer->diagnostics,
            diagnostic_create(position, format_string("invalid number literal: '%s'", string_value))
        );

        return TOKEN_INVALID;
    }

    if (is_float) {
        return (Token){
            .type = TOKEN_TYPE_FLOAT_LITERAL,
            .position = position,
            .number = value,
        };
    } else {
        return (Token){
            .type = TOKEN_TYPE_INTEGER_LITERAL,
            .position = position,
            .integer = (uint64_t)value,
        };
    }
}

bool lexer_is_eof(Lexer* lexer) {
    // The lexer is considered to be at the end of the file if there are no characters left.
    return lexer->position.index >= lexer->contents.length;
}

char lexer_peek(Lexer* lexer) {
    return lexer->contents.data[lexer->position.index];
}

char lexer_consume(Lexer* lexer) {
    auto character = lexer_peek(lexer);

    // We must advance the index and the column.
    lexer->position.index++;
    lexer->position.column++;

    return character;
}

void lexer_destroy(Lexer lexer) {
    file_contents_destroy(lexer.contents);
}
