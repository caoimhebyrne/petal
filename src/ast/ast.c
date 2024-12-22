#include "ast.h"

bool ast_initialize(AST* ast, TokenStream token_stream) {
    ast->diagnostics = (DiagnosticStream){};
    ast->token_stream = token_stream;

    return diagnostic_stream_initialize(&ast->diagnostics, 2);
}
