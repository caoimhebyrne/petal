#include "lexer.h"
#include "allocator.h"
#include "array.h"
#include "logger.h"
#include <assert.h>
#include <ctype.h>
#include <stdlib.h>

IMPLEMENT_ARRAY_TYPE(TokenArray, token_array, Token)

void lexer_init(Lexer* lexer, Allocator* allocator, const StringBuffer* buffer) {
    assert(buffer != NULL && "Invalid buffer passed to lexer_init");

    lexer->allocator = allocator;
    lexer->buffer = buffer;
    lexer->cursor = 0;
}

char lexer_peek(const Lexer* lexer) { return lexer->buffer->data[lexer->cursor]; }

char lexer_consume(Lexer* lexer) { return lexer->buffer->data[lexer->cursor++]; }

bool lexer_is_identifier(const Lexer* lexer) {
    const char character = lexer_peek(lexer);
    return character == '_' || isalnum(character);
}

bool lexer_parse_identifier(Lexer* lexer, TokenArray* tokens);
bool lexer_parse_number(Lexer* lexer, TokenArray* tokens);

bool lexer_parse(Lexer* lexer, TokenArray* tokens) {
    while (lexer->cursor < lexer->buffer->length) {
        const char character = lexer_peek(lexer);

        switch (character) {
        case ' ':
        case '\n':
            lexer_consume(lexer);
            continue;

        default:
            if (isdigit(character)) {
                if (!lexer_parse_number(lexer, tokens))
                    return false;
            } else if (lexer_is_identifier(lexer)) {
                if (!lexer_parse_identifier(lexer, tokens))
                    return false;
            } else {
                log_error("unrecognized character: '%c'", character);
                return false;
            }

            continue;
        }
    }

    return true;
}

bool lexer_parse_identifier(Lexer* lexer, TokenArray* tokens) {
    StringBuffer identifier = {0};
    string_buffer_init(&identifier, lexer->allocator);

    while (lexer_is_identifier(lexer)) {
        string_buffer_append(&identifier, lexer_consume(lexer));
    }

    // C strings are null-terminated.
    string_buffer_append(&identifier, '\0');
    token_array_append(tokens, (Token){.kind = TOKEN_KIND_IDENTIFIER, .string = identifier.data});

    return true;
}

bool lexer_parse_number(Lexer* lexer, TokenArray* tokens) {
    StringBuffer number = {0};
    string_buffer_init(&number, lexer->allocator);

    do {
        const char character = lexer_peek(lexer);

        if (!isdigit(character) && character != '.' && character != 'e' && character != 'E')
            break;

        string_buffer_append(&number, lexer_consume(lexer));
    } while (true);

    // C strings are null-terminated.
    string_buffer_append(&number, '\0');

    char* end_pointer = number.data;
    const float value = strtod(number.data, &end_pointer);

    // The end pointer should be set to the character *before* the null byte.
    if (end_pointer != number.data + number.length - 1) {
        log_error("failed to parse '%.*s' as a valid number", (int)number.length, number.data);
        return false;
    }

    token_array_append(tokens, (Token){.kind = TOKEN_KIND_NUMBER, .number = value});
    return true;
}
