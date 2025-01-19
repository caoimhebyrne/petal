#include "ast/node/type_declaration.h"
#include "ast/node.h"
#include "core/type/type.h"
#include "util/defer.h"
#include "util/format.h"
#include <stdlib.h>

TypeDeclarationNode* type_declaration_node_create(Position position, char* name, Type* type) {
    TypeDeclarationNode* node = malloc(sizeof(TypeDeclarationNode));
    if (!node) {
        return nullptr;
    }

    node->header.kind = NODE_KIND_TYPE_DECLARATION;
    node->header.position = position;
    node->name = name;
    node->type = type;

    return node;
}

char* type_declaration_node_to_string(TypeDeclarationNode* node) {
    auto type_string defer(free_str) = type_to_string(node->type);
    return format_string("TypeDeclarationNode { name = '%s', type = %s }", node->name, type_string);
}

void type_declaration_node_destroy(TypeDeclarationNode* node) {
    free(node->name);
    type_destroy(node->type);
}
