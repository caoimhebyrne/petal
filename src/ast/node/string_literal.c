#include "string_literal.h"
#include "../../string/format_string.h"

StringLiteralNode* string_literal_node_create(Position position, char* value) {
    StringLiteralNode* node = malloc(sizeof(StringLiteralNode));
    node->node_type = NODE_STRING_LITERAL;
    node->position = position;
    node->value = value;

    return node;
}

char* string_literal_node_to_string(StringLiteralNode* node) {
    return format_string("string literal ('%s')", node->value);
}
