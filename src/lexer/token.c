#include "lexer/token.h"
#include <stdlib.h>

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
