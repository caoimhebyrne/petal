#include "ast/ast.h"
#include "ast/node.h"
#include "ast/node/function_declaration.h"
#include "codegen/codegen.h"
#include "diagnostics.h"
#include "lexer/lexer.h"
#include "lexer/token.h"
#include "logger.h"
#include <stdio.h>
#include <stdlib.h>

void print_node_stream(NodeStream node_stream, int depth) {
    for (size_t i = 0; i < node_stream.length; i++) {
        Node* node = node_stream.data[i];
        printf("%*c- %s\n", depth, ' ', node_to_string(node));

        if (node->node_type == NODE_FUNCTION_DECLARATION) {
            FunctionDeclarationNode* function_node = (FunctionDeclarationNode*)node;
            print_node_stream(function_node->function_body, depth + 4);
        }
    }
}

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
    if (lexer.diagnostics.length != 0) {
        token_stream_destroy(&token_stream);
        diagnostic_stream_print(&lexer.diagnostics, filename);
        lexer_destroy(&lexer);

        return -1;
    }

    LOG_INFO("main", "parsed %zu token(s)", token_stream.length);

    AST ast;
    if (!ast_initialize(&ast, token_stream)) {
        return -1;
    }

    NodeStream node_stream = ast_parse(&ast);
    if (ast.diagnostics.length != 0) {
        node_stream_destroy(&node_stream);
        diagnostic_stream_print(&ast.diagnostics, filename);
        ast_destroy(&ast);

        return -1;
    }

    ast_destroy(&ast);

    Codegen codegen = codegen_create(node_stream);

    char* code = codegen_generate(&codegen);

    FILE* output = fopen("./build/output.c", "w");
    fprintf(output, "%s", code);
    fclose(output);

    if (system("cc -o ./build/output ./build/output.c") != 0) {
        LOG_ERROR("main", "failed to compile!");
        return -1;
    }

    LOG_INFO("main", "compiled to ./build/output");

    node_stream_destroy(&node_stream);

    return 0;
}
