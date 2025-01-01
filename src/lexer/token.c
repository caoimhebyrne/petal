#include "lexer/token.h"
#include <stdlib.h>

void token_destroy(Token token) {
    if (token.string) {
        free(token.string);
    }
}
