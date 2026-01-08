#include "lexer.h"
#include "array.h"
#include <assert.h>

IMPLEMENT_ARRAY_TYPE(TokenArray, token_array, Token)

void lexer_init(Lexer* lexer, const StringBuffer *buffer) {
    assert(buffer != NULL && "Invalid buffer passed to lexer_init");
    
    lexer->buffer = buffer;
    lexer->cursor = 0;
}

bool lexer_parse(Lexer *lexer, TokenArray *tokens) {
    (void)lexer;
    (void)tokens;

    return false;
}
