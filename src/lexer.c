#include "lexer.h"
#include "array.h"
#include "logger.h"
#include "module.h"
#include <assert.h>
#include <ctype.h>
#include <stdlib.h>

IMPLEMENT_ARRAY_TYPE(TokenArray, token_array, Token)

void lexer_init(Lexer* lexer, const Module* module) {
    assert(module->source.data != NULL && "Invalid buffer passed to lexer_init");

    lexer->allocator = module->allocator;
    lexer->module_id = module->id;
    lexer->buffer = &module->source;
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
        case '=':
            lexer_consume(lexer);
            token_array_append(tokens, (Token){.kind = TOKEN_KIND_EQUALS});
            continue;

        case '(':
            lexer_consume(lexer);
            token_array_append(tokens, (Token){.kind = TOKEN_KIND_OPEN_PARENTHESIS});
            continue;

        case ')':
            lexer_consume(lexer);
            token_array_append(tokens, (Token){.kind = TOKEN_KIND_CLOSE_PARENTHESIS});
            continue;

        case '{':
            lexer_consume(lexer);
            token_array_append(tokens, (Token){.kind = TOKEN_KIND_OPEN_BRACE});
            continue;

        case '}':
            lexer_consume(lexer);
            token_array_append(tokens, (Token){.kind = TOKEN_KIND_CLOSE_BRACE});
            continue;

        case ':':
            lexer_consume(lexer);
            token_array_append(tokens, (Token){.kind = TOKEN_KIND_COLON});
            continue;

        case ';':
            lexer_consume(lexer);
            token_array_append(tokens, (Token){.kind = TOKEN_KIND_SEMICOLON});
            continue;

        case ',':
            lexer_consume(lexer);
            token_array_append(tokens, (Token){.kind = TOKEN_KIND_COMMA});
            continue;

        case '>':
            lexer_consume(lexer);
            token_array_append(tokens, (Token){.kind = TOKEN_KIND_RIGHT_ANGLE_BRACKET});
            continue;

        case '-':
            lexer_consume(lexer);
            token_array_append(tokens, (Token){.kind = TOKEN_KIND_HYPHEN});
            continue;

        case '/':
            lexer_consume(lexer);

            if (lexer_peek(lexer) == '/') {
                lexer_consume(lexer);

                while (lexer_peek(lexer) != '\n') {
                    lexer_consume(lexer);
                    continue;
                }
            } else {
                token_array_append(tokens, (Token){.kind = TOKEN_KIND_SLASH});
            }

            continue;

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
    const float value = strtof(number.data, &end_pointer);

    // The end pointer should be set to the character *before* the null byte.
    if (end_pointer != number.data + number.length - 1) {
        log_error("failed to parse '%.*s' as a valid number", (int)number.length, number.data);
        return false;
    }

    token_array_append(tokens, (Token){.kind = TOKEN_KIND_NUMBER, .number = value});
    return true;
}
