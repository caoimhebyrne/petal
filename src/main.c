#include "lexer/lexer.h"
#include "lexer/token.h"
#include "logger.h"
#include <stdio.h>

int main(int argc, char** argv) {
    // FIXME: Only one filename is supported right now.
    if (argc != 2) {
        fprintf(stderr, "Usage: %s [FILE]\n", argv[0]);
        return -1;
    }

    Lexer lexer;
    if (!lexer_initialize(&lexer, argv[1])) {
        return -1;
    }

    TokenStream stream = lexer_parse(&lexer);
    if (stream.length == 0) {
        token_stream_destroy(&stream);
        return -1;
    }

    for (size_t i = 0; i < stream.length; i++) {
        Token token = stream.data[i];
        LOG_INFO("main", "token %zu: %s", i, token_to_string(&token));
    }

    token_stream_destroy(&stream);
    lexer_destroy(&lexer);

    return 0;
}
