#include "ast/node/number_literal.h"
#include "util/format.h"
#include <stdint.h>
#include <stdlib.h>

NumberLiteralNode* number_literal_node_create_float(Position position, double value) {
    NumberLiteralNode* node = malloc(sizeof(NumberLiteralNode));
    if (!node) {
        return nullptr;
    }

    node->header.kind = NODE_KIND_NUMBER_LITERAL;
    node->header.position = position;
    node->is_float = true;
    node->type = nullptr;
    node->number = value;

    return node;
}

NumberLiteralNode* number_literal_node_create_integer(Position position, uint64_t value) {
    NumberLiteralNode* node = malloc(sizeof(NumberLiteralNode));
    if (!node) {
        return nullptr;
    }

    node->header.kind = NODE_KIND_NUMBER_LITERAL;
    node->header.position = position;
    node->is_float = false;
    node->type = nullptr;
    node->integer = value;

    return node;
}

char* number_literal_node_to_string(NumberLiteralNode* node) {
    if (node->is_float) {
        return format_string("NumberLiteralNode { value = (float) %f }", node->number);
    } else {
        return format_string("NumberLiteralNode { value = (integer) %d }", node->integer);
    }
}

void number_literal_node_destroy(NumberLiteralNode* node) {
    if (node->type) {
        type_destroy(node->type);
    }
}
