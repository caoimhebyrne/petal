#include "number_literal.h"
#include "../../string/format_string.h"
#include <stdlib.h>

NumberLiteralNode* number_literal_node_create(Position position, double value) {
    NumberLiteralNode* node = malloc(sizeof(NumberLiteralNode));
    if (!node) {
        return 0;
    }

    node->node_type = NODE_NUMBER_LITERAL;
    node->position = position;
    node->value = value;
    node->expected_type = TYPE_INVALID;

    return node;
}

char* number_literal_node_to_string(NumberLiteralNode* node) {
    return format_string("number literal (value: %f)", node->value);
}
