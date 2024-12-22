#include "diagnostics.h"
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

    char* filename = argv[1];

    Lexer lexer;
    if (!lexer_initialize(&lexer, filename)) {
        return -1;
    }

    TokenStream token_stream = lexer_parse(&lexer);
    lexer_destroy(&lexer);

    if (lexer.diagnostics.length != 0) {
        token_stream_destroy(&token_stream);
        diagnostic_stream_print(&lexer.diagnostics, filename);

        return -1;
    }

    for (size_t i = 0; i < token_stream.length; i++) {
        Token token = token_stream.data[i];
        LOG_INFO("main", "token %zu: %s", i, token_to_string(&token));
    }

    token_stream_destroy(&token_stream);

    return 0;
}
