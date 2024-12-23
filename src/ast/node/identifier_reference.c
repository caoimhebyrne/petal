#include "identifier_reference.h"
#include "../../string/format_string.h"
#include <stdlib.h>

IdentifierReferenceNode* identifier_reference_node_create(Position position, char* name) {
    IdentifierReferenceNode* node = malloc(sizeof(IdentifierReferenceNode));
    if (!node) {
        return 0;
    }

    node->node_type = NODE_IDENTIFIER_REFERENCE;
    node->position = position;
    node->name = name;

    return node;
}

char* identifier_reference_node_to_string(IdentifierReferenceNode* node) {
    return format_string("identifier reference ('%s')", node->name);
}
