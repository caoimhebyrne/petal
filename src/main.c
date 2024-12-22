#include "ast/ast.h"
#include "diagnostics.h"
#include "lexer/lexer.h"
#include "lexer/token.h"
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

    AST ast;
    if (!ast_initialize(&ast, token_stream)) {
        return -1;
    }

    token_stream_destroy(&token_stream);

    return 0;
}
