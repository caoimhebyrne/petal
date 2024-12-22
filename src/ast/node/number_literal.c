#include "number_literal.h"
#include "../../string/format_string.h"
#include <stdlib.h>

NumberLiteralNode* number_literal_node_create(double value) {
    NumberLiteralNode* node = malloc(sizeof(NumberLiteralNode));
    node->node_type = NODE_NUMBER_LITERAL;
    node->value = value;

    return node;
}

char* number_literal_node_to_string(NumberLiteralNode* node) {
    return format_string("number literal (value: %f)", node->value);
}
