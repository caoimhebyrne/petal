#include "boolean_literal.h"
#include "../../string/format_string.h"
#include <stdlib.h>

BooleanLiteralNode* boolean_literal_node_create(Position position, bool value) {
    BooleanLiteralNode* node = malloc(sizeof(BooleanLiteralNode));
    if (!node) {
        return 0;
    }

    node->node_type = NODE_BOOLEAN_LITERAL;
    node->position = position;
    node->value = value;

    return node;
}

char* boolean_literal_node_to_string(BooleanLiteralNode* node) {
    return format_string("boolean literal (value: %s)", node->value ? "true" : "false");
}
