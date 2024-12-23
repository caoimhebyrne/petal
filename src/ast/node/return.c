#include "return.h"
#include "../../string/format_string.h"
#include <stdlib.h>

ReturnNode* return_node_create(Position position, Node* value) {
    ReturnNode* node = malloc(sizeof(ReturnNode));
    if (!node) {
        return 0;
    }

    node->node_type = NODE_RETURN;
    node->position = position;
    node->value = value;

    return node;
}

char* return_node_to_string(ReturnNode* node) {
    if (node->value == 0) {
        return "return statement (no value)";
    }

    return format_string("return statement (value: %s)", node_to_string(node->value));
}
