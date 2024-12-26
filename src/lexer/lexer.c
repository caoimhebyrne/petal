#include "lexer.h"
#include "../logger.h"
#include "../string/format_string.h"
#include "../string/string_builder.h"
#include "token.h"
#include <ctype.h>
#include <errno.h> // IWYU pragma: keep
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/stat.h>

char* keywords[] = {"func", "extern", "return", "true", "false"};
size_t keywords_length = sizeof(keywords) / sizeof(char*);

bool lexer_initialize(Lexer* lexer, char* filename) {
    FILE* file = fopen(filename, "r");
    if (!file) {
        LOG_ERROR("lexer", "failed to open file %s: %s", filename, strerror(errno));
        return false;
    }

    // In order to use the `stat` API, we need to get the file descriptor for the opened file.
    int file_descriptor = fileno(file);

    struct stat stat_result;
    if (fstat(file_descriptor, &stat_result) != 0) {
        LOG_ERROR("lexer", "failed to read file %s: %s", filename, strerror(errno));
        return false;
    }

    // Ensure that the filename passed is not actually a directory.
    if (S_ISDIR(stat_result.st_mode)) {
        LOG_ERROR("lexer", "%s is a directory, expected a file", filename);
        return 0;
    }

    size_t file_size = stat_result.st_size;

    char* contents = malloc(file_size);
    if (!contents) {
        LOG_ERROR("lexer", "failed to allocate buffer of %zu for file %s", file_size, filename);
        return false;
    }

    if (fread(contents, file_size, sizeof(char), file) != 1) {
        LOG_ERROR("lexer", "failed to read %zu bytes from %s: %s", file_size, filename, strerror(errno));
        return false;
    }

    fclose(file);

    lexer->contents = contents;
    lexer->contents_length = file_size;
    lexer->position = (Position){.line = 0, .column = 0, .index = 0};
    lexer->diagnostics = (DiagnosticStream){};
    diagnostic_stream_initialize(&lexer->diagnostics, 2);

    LOG_DEBUG("lexer", "initialized lexer with %zu bytes from %s", file_size, filename);

    return true;
}

TokenStream lexer_parse(Lexer* lexer) {
    TokenStream stream;
    token_stream_initialize(&stream, 2);

    for (; lexer->position.index < lexer->contents_length; position_advance(&lexer->position)) {
        char character = lexer->contents[lexer->position.index];
        switch (character) {

        case '/': {
            if (lexer->contents[lexer->position.index + 1] == '/') {
                // If this is a comment (//), skip until the next new-line.
                while (lexer->position.index < lexer->contents_length) {
                    position_advance(&lexer->position);

                    // NOTE: This also works for carraige returns, \r is skipped as part of the comment.
                    if (lexer->contents[lexer->position.index] == '\n') {
                        position_advance_line(&lexer->position);
                        break;
                    }
                }
            } else {
                // This is a single `/`, not a comment.
                token_stream_append(&stream, (Token){.type = TOKEN_SLASH});
            }

            break;
        }

        // Ignore any whitespace!
        case ' ':
        case '\t':
        case '\r':
            break;

        case '\n':
            position_advance_line(&lexer->position);
            break;

        case '=':
            token_stream_append(&stream, (Token){.type = TOKEN_EQUALS, .position = lexer->position});
            break;

        case ';':
            token_stream_append(&stream, (Token){.type = TOKEN_SEMICOLON, .position = lexer->position});
            break;

        case '(':
            token_stream_append(&stream, (Token){.type = TOKEN_OPEN_PARENTHESIS, .position = lexer->position});
            break;

        case ')':
            token_stream_append(&stream, (Token){.type = TOKEN_CLOSE_PARENTHESIS, .position = lexer->position});
            break;

        case '{':
            token_stream_append(&stream, (Token){.type = TOKEN_OPEN_BRACE, .position = lexer->position});
            break;

        case '}':
            token_stream_append(&stream, (Token){.type = TOKEN_CLOSE_BRACE, .position = lexer->position});
            break;

        case '*':
            token_stream_append(&stream, (Token){.type = TOKEN_ASTERISK, .position = lexer->position});
            break;

        case '-':
            token_stream_append(&stream, (Token){.type = TOKEN_HYPHEN, .position = lexer->position});
            break;

        case '>':
            token_stream_append(&stream, (Token){.type = TOKEN_RIGHT_ANGLE_BRACKET, .position = lexer->position});
            break;

        case ':':
            token_stream_append(&stream, (Token){.type = TOKEN_COLON, .position = lexer->position});
            break;

        case ',':
            token_stream_append(&stream, (Token){.type = TOKEN_COMMA, .position = lexer->position});
            break;

        case '+':
            token_stream_append(&stream, (Token){.type = TOKEN_PLUS, .position = lexer->position});
            break;

        case '"': {
            lexer->position.index += 1;
            Token token = lexer_parse_string_literal(lexer);
            if (token.type != TOKEN_INVALID) {
                token_stream_append(&stream, token);
                continue;
            }

            break;
        }

        default: {
            if (isalpha(character)) {
                // If the character is an alphabetic character, attempt to parse an identifier.
                Token token = lexer_parse_identifier(lexer);
                if (token.type != TOKEN_INVALID) {
                    token_stream_append(&stream, token);
                    continue;
                }
            } else if (isdigit(character)) {
                // If the character is a digit, attempt to parse a number literal.
                Token token = lexer_parse_number_literal(lexer);
                if (token.type != TOKEN_INVALID) {
                    token_stream_append(&stream, token);
                    continue;
                }
            }

            Diagnostic diagnostic = {
                .position = lexer->position,
                .message = format_string("unknown character: '%c'", character),
                .is_terminal = true,
            };

            diagnostic_stream_append(&lexer->diagnostics, diagnostic);
            break;
        }
        }
    }

    return stream;
}

Token lexer_parse_identifier(Lexer* lexer) {
    StringBuilder string_builder;
    if (!string_builder_initialize(&string_builder, 2)) {
        return INVALID_TOKEN;
    }

    Position starting_position = lexer->position;
    for (; lexer->position.index < lexer->contents_length; position_advance(&lexer->position)) {
        char character = lexer->contents[lexer->position.index];
        if (!isalpha(character) && !isdigit(character) && character != '_') {
            position_retreat(&lexer->position); // Don't consume the character, it is not part of the identifier.
            break;
        }

        string_builder_append(&string_builder, character);
    }

    TokenType token_type = TOKEN_IDENTIFIER;
    char* identifier_name = string_builder_finish(&string_builder);

    for (size_t i = 0; i < keywords_length; i++) {
        // TODO: Hashtable
        char* keyword = keywords[i];
        if (strcmp(identifier_name, keyword) == 0) {
            token_type = TOKEN_KEYWORD;
            break;
        }
    }

    return (Token){.type = token_type, .string = identifier_name, .position = starting_position};
}

Token lexer_parse_number_literal(Lexer* lexer) {
    StringBuilder string_builder;
    if (!string_builder_initialize(&string_builder, 2)) {
        return INVALID_TOKEN;
    }

    Position starting_position = lexer->position;
    for (; lexer->position.index < lexer->contents_length; position_advance(&lexer->position)) {
        char character = lexer->contents[lexer->position.index];
        if (!isdigit(character) && character != '.') {
            position_retreat(&lexer->position); // Don't consume the character, it is not part of the number.
            break;
        }

        string_builder_append(&string_builder, character);
    }

    char* string_value = string_builder_finish(&string_builder);

    // strtod will set end_ptr to the first character after the numeric value has been parsed.
    char* end_ptr = string_value;
    double value = strtod(string_value, &end_ptr);

    // If the end_ptr is still equal to the string_value, this is an invalid number.
    if (end_ptr == string_value) {
        LOG_ERROR("lexer", "invalid number: '%s'", string_value);
        return INVALID_TOKEN;
    }

    return (Token){.type = TOKEN_NUMBER_LITERAL, .number = value, .position = starting_position};
}

Token lexer_parse_string_literal(Lexer* lexer) {
    StringBuilder string_builder;
    if (!string_builder_initialize(&string_builder, 2)) {
        return INVALID_TOKEN;
    }

    Position starting_position = lexer->position;
    for (; lexer->position.index < lexer->contents_length; position_advance(&lexer->position)) {
        char character = lexer->contents[lexer->position.index];
        if (character == '\n' || character == '"') {
            break;
        }

        // FIXME: We need a better way to parse escape sequences like \n.
        //        Also need to be able to escape them with an extra \.
        if (character == '\\' && lexer->contents[lexer->position.index + 1] == 'n') {
            string_builder_append(&string_builder, '\n');
            position_advance(&lexer->position);
        } else {
            string_builder_append(&string_builder, character);
        }
    }

    char* value = string_builder_finish(&string_builder);
    return (Token){.type = TOKEN_STRING_LITERAL, .string = value, .position = starting_position};
}

void lexer_destroy(Lexer* lexer) {
    free(lexer->contents);
    diagnostic_stream_destroy(&lexer->diagnostics);
}
