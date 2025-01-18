#include "ast/node/variable_reassignment.h"
#include "ast/node.h"
#include "util/defer.h"
#include "util/format.h"
#include <stdlib.h>

VariableReassignmentNode* variable_reassignment_node_create(Position position, char* name, Node* value) {
    VariableReassignmentNode* node = malloc(sizeof(VariableReassignmentNode));
    if (!node) {
        return nullptr;
    }

    node->header.kind = NODE_KIND_VARIABLE_REASSIGNMENT;
    node->header.position = position;
    node->name = name;
    node->value = value;

    return node;
}

char* variable_reassignment_node_to_string(VariableReassignmentNode* node) {
    auto value_string defer(free_str) = node_to_string(node->value);
    return format_string("VariableReassignmentNode { name = '%s', value = %s }", node->name, value_string);
}

void variable_reassignment_node_destroy(VariableReassignmentNode* node) {
    free(node->name);
    node_destroy(node->value);
}
