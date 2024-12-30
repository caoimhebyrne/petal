#include "variable_reassignment.h"
#include "../../string/format_string.h"
#include <stdlib.h>

VariableReassignmentNode* variable_reassignment_node_create(Position position, char* variable_name, Node* value) {
    VariableReassignmentNode* node = malloc(sizeof(VariableReassignmentNode));
    if (!node) {
        return 0;
    }

    node->node_type = NODE_VARIABLE_REASSIGNMENT;
    node->position = position;
    node->variable_name = variable_name;
    node->value = value;

    return node;
}

char* variable_reassignment_node_to_string(VariableReassignmentNode* node) {
    return format_string("variable reassignment (name: '%s', value: '%s')", node->variable_name,
                         node_to_string(node->value));
}
