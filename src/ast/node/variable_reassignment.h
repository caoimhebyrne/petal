#pragma once

#include "ast/node.h"

typedef struct {
    union {
        Node header;
    };

    // The variable being re-assigned.
    char* name;

    // The value being assigned to the variable.
    Node* value;
} VariableReassignmentNode;

// Creates a new VariableReassignmentNode.
// Parameters:
// - position: The position that this node occurred at within the source file.
// - name: The name of the variable.
// - value: The value being assigned to the variable.
VariableReassignmentNode* variable_reassignment_node_create(Position position, char* name, Node* value);

// Returns a string representation of a VariableReassignmentNode.
char* variable_reassignment_node_to_string(VariableReassignmentNode* node);

// De-allocates an VariableReassignmentNode's data.
// Parmaeters:
// - node: The VariableReassignmentNode to destroy.
void variable_reassignment_node_destroy(VariableReassignmentNode* node);
