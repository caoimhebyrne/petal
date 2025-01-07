#include "ast/node/return.h"
#include "ast/node.h"
#include "util/defer.h"
#include "util/format.h"
#include <stdlib.h>

ReturnNode* return_node_create(Position position, Node* return_value) {
    ReturnNode* node = malloc(sizeof(ReturnNode));
    if (!node) {
        return nullptr;
    }

    node->header.kind = NODE_KIND_RETURN;
    node->header.position = position;
    node->return_value = return_value;

    return node;
}

// Returns a string representation of an ReturnNode.
char* return_node_to_string(ReturnNode* node) {
    if (node->return_value) {
        auto value_string defer(free_str) = node_to_string(node->return_value);
        return format_string("ReturnNode { value = %s }", value_string);
    } else {
        return format_string("ReturnNode");
    }
}

// De-allocates an ReturnNode's data.
// Parmaeters:
// - node: The ReturnNode to destroy.
void return_node_destroy(ReturnNode* node) {
    if (node->return_value) {
        node_destroy(node->return_value);
    }
}
