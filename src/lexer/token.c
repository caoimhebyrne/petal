#include "token.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

void token_stream_destroy(TokenStream* stream) { free(stream->data); }

char* token_to_string(Token* token) {
    switch (token->type) {
    case TOKEN_INVALID:
        return "invalid token";

    case TOKEN_IDENTIFIER: {
        const char* format = "identifier ('%s')";
        size_t length = snprintf(NULL, 0, format, token->string);

        char* buffer = malloc(length + 1);
        if (!buffer) {
            return "identifier (?)";
        }

        snprintf(buffer, length + 1, format, token->string);
        return buffer;
    }

    case TOKEN_NUMBER_LITERAL: {
        const char* format = "number literal ('%f')";
        size_t length = snprintf(NULL, 0, format, token->number);

        char* buffer = malloc(length);
        if (!buffer) {
            return "number literal (?)";
        }

        snprintf(buffer, length + 1, format, token->number);
        return buffer;
    }

    case TOKEN_EQUALS:
        return "equals ('=')";

    case TOKEN_SEMICOLON:
        return "semicolon (';')";

    case TOKEN_SLASH:
        return "slash ('/')";

    case TOKEN_OPEN_PARENTHESIS:
        return "open parenthesis ('(')";

    case TOKEN_CLOSE_PARENTHESIS:
        return "close parenthesis (')')";

    case TOKEN_OPEN_BRACE:
        return "open brace ('{')";

    case TOKEN_CLOSE_BRACE:
        return "close brace ('}')";
    }

    return "unknown";
}

CREATE_STREAM(TokenStream, token_stream, Token);
