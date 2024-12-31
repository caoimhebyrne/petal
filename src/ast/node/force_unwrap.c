#include "force_unwrap.h"
#include "../../string/format_string.h"
#include <stdlib.h>

ForceUnwrapNode* force_unwrap_node_create(Position position, Node* value) {
    ForceUnwrapNode* node = malloc(sizeof(ForceUnwrapNode));
    if (!node) {
        return 0;
    }

    node->node_type = NODE_FORCE_UNWRAP;
    node->position = position;
    node->value = value;

    return node;
}

char* force_unwrap_node_to_string(ForceUnwrapNode* node) {
    return format_string("force unwrap (value: '%s')", node_to_string(node->value));
}
