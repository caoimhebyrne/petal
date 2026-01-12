#include "lexer.h"
#include "array.h"
#include "logger.h"
#include "module.h"
#include <assert.h>
#include <ctype.h>
#include <stdlib.h>
#include <string.h>

IMPLEMENT_ARRAY_TYPE(TokenArray, token_array, Token)

void lexer_init(Lexer* lexer, const Module* module) {
    assert(module->source.data != NULL && "Invalid buffer passed to lexer_init");

    lexer->allocator = module->allocator;
    lexer->module_id = module->id;
    lexer->buffer = &module->source;
    lexer->cursor = 0;
    lexer->position = (Position){
        .line = 0,
        .column = 0,
        .length = 0,
        .module_id = module->id,
    };
}

// Returns whether the lexer has reached the end of the source code or not.
bool lexer_is_eof(const Lexer* lexer) { return lexer->cursor >= lexer->buffer->length; }

// Returns the character at the lexer's current position, without advancing the iterator.
char lexer_peek(const Lexer* lexer) {
    if (lexer_is_eof(lexer)) {
        return 0;
    }

    return lexer->buffer->data[lexer->cursor];
}

// Returns the character at the lexer's current position (plus a certain offset), without advancing the iterator.
char lexer_peek_nth(const Lexer* lexer, const size_t offset) {
    const size_t index = lexer->cursor + offset;
    if (index >= lexer->buffer->length) {
        return 0;
    }

    return lexer->buffer->data[index];
}

// Returns the character at the lexer's current position, advancing the iterator.
char lexer_consume(Lexer* lexer) {
    if (lexer_is_eof(lexer)) {
        return 0;
    }

    const char character = lexer->buffer->data[lexer->cursor++];

    if (character == '\n') {
        lexer->position.line += 1;
        lexer->position.column = 0;
    } else if (lexer->position.length == 0) {
        // If there is no length group ongoing, we can advance the column.
        lexer->position.column += 1;
    } else {
        // Otherwise, we must advance the length by 1.
        lexer->position.length += 1;
    }

    return character;
}

// Pushes a token with the provided kind onto the token array, while consuming the current token.
// Returns false if the lexer has reached EOF.
bool lexer_push_single_token(Lexer* lexer, TokenArray* tokens, const TokenKind kind) {
    lexer_consume(lexer);
    token_array_append(tokens, (Token){
                                   .kind = kind,
                                   .position =
                                       (Position){
                                           .line = lexer->position.line,
                                           .column = lexer->position.column,
                                           .length = 1,
                                           .module_id = lexer->position.module_id,
                                       },
                               });
    return true;
}

// Starts a length group at the lexer's current position. This prevents the column from being advanced until
// lexer_end_length_group is called.
void lexer_start_length_group(Lexer* lexer) { lexer->position.length = 1; }

// Ends an ongoing length group.
void lexer_end_length_group(Lexer* lexer) {
    // We can advance the column by the length of the current group.
    lexer->position.column += lexer->position.length;
    lexer->position.length = 0;
}

// Returns whether the character at the lexer's current position is a valid identifier character.
bool lexer_is_identifier(const Lexer* lexer);

// Attempts to parse an identifier token at the lexer's current position.
bool lexer_parse_identifier(Lexer* lexer, TokenArray* tokens);

// Attempts to parse a number token at the lexer's current position.
bool lexer_parse_number(Lexer* lexer, TokenArray* tokens);

bool lexer_parse(Lexer* lexer, TokenArray* tokens) {
    while (!lexer_is_eof(lexer)) {
        // Any ongoing length groups must be ended at the start of each iteration.
        lexer_end_length_group(lexer);

        const char character = lexer_peek(lexer);

        switch (character) {
        case '=':
            lexer_push_single_token(lexer, tokens, TOKEN_KIND_EQUALS);
            continue;

        case '(':
            lexer_push_single_token(lexer, tokens, TOKEN_KIND_OPEN_PARENTHESIS);
            continue;

        case ')':
            lexer_push_single_token(lexer, tokens, TOKEN_KIND_CLOSE_PARENTHESIS);
            continue;

        case '{':
            lexer_push_single_token(lexer, tokens, TOKEN_KIND_OPEN_BRACE);
            continue;

        case '}':
            lexer_push_single_token(lexer, tokens, TOKEN_KIND_CLOSE_BRACE);
            continue;

        case ':':
            lexer_push_single_token(lexer, tokens, TOKEN_KIND_COLON);
            continue;

        case ';':
            lexer_push_single_token(lexer, tokens, TOKEN_KIND_SEMICOLON);
            continue;

        case ',':
            lexer_push_single_token(lexer, tokens, TOKEN_KIND_COMMA);
            continue;

        case '>':
            lexer_push_single_token(lexer, tokens, TOKEN_KIND_RIGHT_ANGLE_BRACKET);
            continue;

        case '-':
            lexer_push_single_token(lexer, tokens, TOKEN_KIND_HYPHEN);
            continue;

        case '/':
            if (lexer_peek_nth(lexer, 1) == '/') {
                while (lexer_peek(lexer) != '\n') {
                    lexer_consume(lexer);
                    continue;
                }
            } else {
                lexer_push_single_token(lexer, tokens, TOKEN_KIND_SLASH);
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

Keyword get_keyword_from_identifier(const StringBuffer* identifier) {
    // TODO: This could be O(1) if we implement string interning, we could then allocate some common keywords by
    //       default and have an instant lookup.
    if (string_buffer_equals_cstr(identifier, "func")) {
        return KEYWORD_FUNC;
    } else if (string_buffer_equals_cstr(identifier, "return")) {
        return KEYWORD_RETURN;
    }

    return KEYWORD_UNKNOWN;
}

bool lexer_is_identifier(const Lexer* lexer) {
    const char character = lexer_peek(lexer);
    return character == '_' || isalnum(character);
}

bool lexer_parse_identifier(Lexer* lexer, TokenArray* tokens) {
    lexer_start_length_group(lexer);

    StringBuffer identifier = {0};
    string_buffer_init(&identifier, lexer->allocator);

    while (lexer_is_identifier(lexer)) {
        string_buffer_append(&identifier, lexer_consume(lexer));
    }

    // If the identifier that was parsed is a keyword, then we can emit that token instead.
    Keyword keyword = get_keyword_from_identifier(&identifier);
    if (keyword != KEYWORD_UNKNOWN) {
        // TODO: Free the StringBuffer, we don't need it anymore.
        token_array_append(tokens, (Token){
                                       .kind = TOKEN_KIND_KEYWORD,
                                       .keyword = keyword,
                                       .position = lexer->position,
                                   });
        return true;
    }

    token_array_append(tokens, (Token){
                                   .kind = TOKEN_KIND_IDENTIFIER,
                                   .string = identifier,
                                   .position = lexer->position,
                               });
    return true;
}

bool lexer_parse_number(Lexer* lexer, TokenArray* tokens) {
    lexer_start_length_group(lexer);

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

    // TODO: Free the StringBuffer as it is no longer needed.
    token_array_append(tokens, (Token){.kind = TOKEN_KIND_NUMBER, .number = value, .position = lexer->position});
    return true;
}
