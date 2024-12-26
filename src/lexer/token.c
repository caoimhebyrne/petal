#include "token.h"
#include "../string/format_string.h"
#include <stdlib.h>

void token_stream_destroy(TokenStream* stream) { free(stream->data); }

char* token_to_string(Token token) {
    switch (token.type) {
    case TOKEN_INVALID:
        return "invalid token";

    case TOKEN_IDENTIFIER:
        return format_string("identifier ('%s')", token.string);

    case TOKEN_KEYWORD:
        return format_string("keyword ('%s')", token.string);

    case TOKEN_NUMBER_LITERAL:
        return format_string("number literal ('%f')", token.number);

    case TOKEN_STRING_LITERAL:
        return format_string("string ('%s')", token.string);

    default:
        return token_type_to_string(token.type);
    }
}

char* token_type_to_string(TokenType token_type) {
    switch (token_type) {
    case TOKEN_IDENTIFIER:
        return "identifier";

    case TOKEN_KEYWORD:
        return "keyword";

    case TOKEN_NUMBER_LITERAL:
        return "number";

    case TOKEN_STRING_LITERAL:
        return "string";

    case TOKEN_INVALID:
        return "invalid token";

    case TOKEN_EQUALS:
        return "equals";

    case TOKEN_SEMICOLON:
        return "semicolon";

    case TOKEN_SLASH:
        return "slash";

    case TOKEN_OPEN_PARENTHESIS:
        return "open parenthesis";

    case TOKEN_CLOSE_PARENTHESIS:
        return "close parenthesis";

    case TOKEN_OPEN_BRACE:
        return "open brace";

    case TOKEN_CLOSE_BRACE:
        return "close brace";

    case TOKEN_ASTERISK:
        return "asterisk";

    case TOKEN_HYPHEN:
        return "hyphen";

    case TOKEN_RIGHT_ANGLE_BRACKET:
        return "right angle bracket";

    case TOKEN_COLON:
        return "colon";

    case TOKEN_COMMA:
        return "comma";

    case TOKEN_PLUS:
        return "plus";
    }

    return "unknown";
}

CREATE_STREAM(TokenStream, token_stream, Token);
