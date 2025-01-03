#include "ast/ast.h"
#include "ast/node.h"
#include "util/vector.h"

AST ast_create(TokenVector tokens) {
    return (AST){
        .tokens = tokens,
        .position = 0,
    };
}

NodeVector ast_parse(AST* ast) {
    NodeVector vector = vector_create();
    if (!vector_initialize(vector, 1)) {
        return vector;
    }

    (void)ast;

    return vector;
}

void ast_destroy(AST ast) {
    vector_destroy(ast.tokens, token_destroy);
}
