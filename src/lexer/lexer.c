#include "lexer/lexer.h"
#include "core/position.h"
#include "lexer/token.h"
#include "util/file.h"
#include "util/string_builder.h"
#include "util/vector.h"
#include <ctype.h>
#include <stdbool.h>
#include <stdio.h>

// Parsing functions:
Token lexer_parse_identifier(Lexer* lexer);

// Forward declarations:
// Whether the lexer has reached the end of the file or not.
bool lexer_is_eof(Lexer* lexer);

// Returns the next character to be parsed, lexer_is_eof should be called before this.
char lexer_peek(Lexer* lexer);

// Consumes a character from the file, advancing the position cursor.
char lexer_consume(Lexer* lexer);

// Produces a single-character token while consuming the current token.
Token lexer_create_token(Lexer* lexer, TokenType token_type);

Lexer lexer_create(FileContents contents) {
    return (Lexer){
        .contents = contents,
        .position = (Position){0},
    };
}

TokenVector lexer_parse(Lexer* lexer) {
    TokenVector vector = vector_create();
    if (!vector_initialize(vector, 2)) {
        return (TokenVector){0};
    }

    // Keep parsing until the lexer reaches the end of the file.
    while (!lexer_is_eof(lexer)) {
        char character = lexer_peek(lexer);
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

        case '=': {
            vector_append(vector, lexer_create_token(lexer, TOKEN_TYPE_EQUALS));
            break;
        }

        default:
            // If the character is an alphabetic character, it is most likely an identifier.
            if (isalpha(character)) {
                Token token = lexer_parse_identifier(lexer);
                if (token.type != TOKEN_TYPE_INVALID) {
                    vector_append(vector, token);
                    continue;
                }
            }

            fprintf(stderr, "error: unknown character: '%c'\n", character);
            vector_destroy(vector, token_destroy);
            return (TokenVector){0};
        }
    }

    return vector;
}

Token lexer_create_token(Lexer* lexer, TokenType token_type) {
    // The current position is the token's position.
    Position position = lexer->position;
    position.length = 1;

    // Advance the cursor.
    lexer_consume(lexer);

    return (Token){
        .type = token_type,
        .position = position,
    };
}

bool lexer_next_is_identifier(Lexer* lexer) {
    char character = lexer_peek(lexer);
    return isalpha(character) || isdigit(character) || character == '_';
}

Token lexer_parse_identifier(Lexer* lexer) {
    // This token starts at the lexer's current position.
    Position position = lexer->position;

    // An identifier must only contain alphanumeric characters or underscores.
    StringBuilder builder = string_builder_create();
    if (string_builder_is_invalid(builder)) {
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
        return TOKEN_INVALID;
    }

    return (Token){
        .type = TOKEN_TYPE_IDENTIFIER,
        .position = position,
        .string = identifier,
    };
}

bool lexer_is_eof(Lexer* lexer) {
    // The lexer is considered to be at the end of the file if there are no characters left.
    return lexer->position.index >= lexer->contents.length;
}

char lexer_peek(Lexer* lexer) {
    return lexer->contents.data[lexer->position.index];
}

char lexer_consume(Lexer* lexer) {
    char character = lexer_peek(lexer);

    // We must advance the index and the column.
    lexer->position.index++;
    lexer->position.column++;

    return character;
}

void lexer_destroy(Lexer lexer) {
    file_contents_destroy(lexer.contents);
}
