#include "typechecker.h"
#include "ast/node/function_declaration.h"
#include "core/diagnostic.h"
#include "util/defer.h"
#include "util/format.h"
#include "util/vector.h"

// Forward declarations:
bool typechecker_check_statement(Typechecker* typechecker, Node* node);
bool typechecker_check_function_declaration(Typechecker* typechecker, FunctionDeclarationNode* node);

ResolvedType* typechecker_resolve_type(Typechecker* typechecker, Type* type);

Typechecker typechecker_create(NodeVector* nodes, DiagnosticVector* diagnostics) {
    return (Typechecker){
        .nodes = nodes,
        .diagnostics = diagnostics,
    };
}

bool typechecker_check(Typechecker* typechecker) {
    for (size_t i = 0; i < typechecker->nodes->length; i++) {
        auto node = vector_get(typechecker->nodes, i);
        if (!typechecker_check_statement(typechecker, node)) {
            return false;
        }
    }

    return true;
}

bool typechecker_check_statement(Typechecker* typechecker, Node* node) {
    switch (node->kind) {
    case NODE_KIND_FUNCTION_DECLARATION:
        return typechecker_check_function_declaration(typechecker, (FunctionDeclarationNode*)node);

    default:
        auto node_string defer(free_str) = node_to_string(node);
        vector_append(
            typechecker->diagnostics,
            diagnostic_create(node->position, format_string("unable to type-check node: '%s'", node_string))
        );

        return false;
    }
}

bool typechecker_check_function_declaration(Typechecker* typechecker, FunctionDeclarationNode* node) {
    // The function's return type must be resolvable.
    auto return_type = typechecker_resolve_type(typechecker, node->return_type);
    if (!return_type) {
        return false;
    }

    // If the return type is OK, we can type check the function's body.
    for (size_t i = 0; i < node->body.length; i++) {
        auto body_node = vector_get(&node->body, i);
        if (!typechecker_check_statement(typechecker, body_node)) {
            return false;
        }
    }

    return true;
}

ResolvedType* typechecker_resolve_type(Typechecker* typechecker, Type* type) {
    (void)typechecker;
    (void)type;

    vector_append(typechecker->diagnostics, diagnostic_create(type->position, format_string("unable to resolve type")));
    return nullptr;
}
