#include "lexer/token.h"
#include "util/format.h"
#include <stdlib.h>

const char* token_type_to_string(TokenType token_type) {
    switch (token_type) {
    case TOKEN_TYPE_INVALID:
        return "invalid";

    case TOKEN_TYPE_IDENTIFIER:
        return "identifier";

    case TOKEN_TYPE_KEYWORD:
        return "keyword";

    case TOKEN_TYPE_INTEGER_LITERAL:
        return "integer literal";

    case TOKEN_TYPE_FLOAT_LITERAL:
        return "float literal";

    case TOKEN_TYPE_EQUALS:
        return "equals";

    case TOKEN_TYPE_COLON:
        return "colon";

    case TOKEN_TYPE_SEMICOLON:
        return "semicolon";

    case TOKEN_TYPE_PLUS:
        return "plus";

    case TOKEN_TYPE_MINUS:
        return "minus";

    case TOKEN_TYPE_ASTERISK:
        return "asterisk";

    case TOKEN_TYPE_SLASH:
        return "slash";

    case TOKEN_TYPE_OPEN_PARENTHESIS:
        return "open parenthesis";

    case TOKEN_TYPE_CLOSE_PARENTHESIS:
        return "close parenthesis";

    case TOKEN_TYPE_RIGHT_ANGLE_BRACKET:
        return "right angle bracket";

    case TOKEN_TYPE_OPEN_BRACE:
        return "open brace";

    case TOKEN_TYPE_CLOSE_BRACE:
        return "close brace";
    }
}

char* token_to_string(Token token) {
    auto type_string = token_type_to_string(token.type);

    switch (token.type) {
    case TOKEN_TYPE_IDENTIFIER:
    case TOKEN_TYPE_KEYWORD:
        return format_string("%s ('%s')", type_string, token.string);

    case TOKEN_TYPE_INTEGER_LITERAL:
        return format_string("%s (%d)", type_string, token.integer);

    case TOKEN_TYPE_FLOAT_LITERAL:
        return format_string("%s (%f)", type_string, token.number);

    default:
        return format_string("%s", type_string);
    }
}

void token_destroy(Token token) {
    switch (token.type) {
    case TOKEN_TYPE_KEYWORD:
    case TOKEN_TYPE_IDENTIFIER:
        free(token.string);
        break;

    default:
        break;
    }
}
