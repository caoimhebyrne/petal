#include "lexer/token.h"
#include "util/vector.h"
#include <stdlib.h>

void token_vector_destroy(Vector* vector) {
    for (size_t i = 0; i < vector->size; i++) {
        Token* token = vector->items[i];
        token_destroy(*token);
    }

    vector_destroy(vector);
}

void token_destroy(Token token) {
    if (token.string) {
        free(token.string);
    }
}
