#include "ast/ast.h"
#include "ast/node.h"
#include "codegen/llvm_codegen.h"
#include "diagnostics.h"
#include "lexer/lexer.h"
#include "lexer/token.h"
#include "logger.h"
#include <stdlib.h>

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

    LLVMCodegen codegen = llvm_codegen_create(filename, node_stream);
    llvm_codegen_generate(&codegen);

    if (codegen.diagnostics.length != 0) {
        diagnostic_stream_print(&codegen.diagnostics, filename);
        llvm_codegen_destroy(&codegen);

        return -1;
    }

    char* error_message = llvm_codegen_emit(&codegen, "./build/output.o");
    if (error_message) {
        LOG_ERROR("main", "%s", error_message);
        return -1;
    }

    int linker_status = system("gcc -fuse-ld=lld ./build/output.o -o ./build/output");
    if (linker_status != 0) {
        LOG_INFO("main", "linker failed! (%d)", linker_status);
        return -1;
    }

    llvm_codegen_destroy(&codegen);
    return 0;
}
