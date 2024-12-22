#include "ast/ast.h"
#include "ast/node.h"
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

    LOG_INFO("main", "parsed %zu token(s)", token_stream.length);

    AST ast;
    if (!ast_initialize(&ast, token_stream)) {
        return -1;
    }

    NodeStream node_stream = ast_parse(&ast);
    token_stream_destroy(&token_stream);

    if (ast.diagnostics.length != 0) {
        diagnostic_stream_print(&ast.diagnostics, filename);
        return -1;
    }

    LOG_INFO("main", "parsed %zu node(s)", node_stream.length);

    for (size_t i = 0; i < node_stream.length; i++) {
        LOG_INFO("main", "node %zu: %s", i, node_to_string(node_stream.data[i]));
    }

    node_stream_destroy(&node_stream);

    return 0;
}
