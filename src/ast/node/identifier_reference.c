#include "identifier_reference.h"
#include "ast/node.h"
#include "core/position.h"
#include "core/type/type.h"
#include "util/format.h"
#include <stdlib.h>

IdentifierReferenceNode* identifier_reference_node_create(Position position, char* identifier, Type* value_type) {
    IdentifierReferenceNode* node = malloc(sizeof(IdentifierReferenceNode));
    if (!node) {
        return nullptr;
    }

    node->header.kind = NODE_KIND_IDENTIFIER_REFERENCE;
    node->header.position = position;
    node->identifier = identifier;
    node->value_type = value_type;

    return node;
}

char* identifier_reference_node_to_string(IdentifierReferenceNode* node) {
    return format_string("IdentifierReferenceNode { identifier = '%s' }", node->identifier);
}

void identifier_reference_node_destroy(IdentifierReferenceNode* node) {
    free(node->identifier);
    if (node->value_type) {
        type_destroy(node->value_type);
    }
}
