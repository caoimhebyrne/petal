#ifndef __NODE_VARIABLE_REASSIGNMENT_H__
#define __NODE_VARIABLE_REASSIGNMENT_H__

#include "../node.h"

typedef struct {
    // The type of this node, always NODE_VARIABLE_REASSIGNMENT.
    NodeType node_type;

    // The position that this node occurred within the source file.
    Position position;

    // The name of the variable that is receiving the value.
    char* variable_name;

    // The value to reassign into the variable.
    Node* value;
} VariableReassignmentNode;

// Creates a new variable reassignment node.
// Parameters:
// - position: The position that this node occurred at within the source file.
// - variable_name: The name of the variable that is receiving the value.
// - value: The value to reassign into the variable.
VariableReassignmentNode* variable_reassignment_node_create(Position position, char* variable_name, Node* value);

// Returns a string representation of a variable reassignment node.
char* variable_reassignment_node_to_string(VariableReassignmentNode* node);

#endif // __NODE_VARIABLE_REASSIGNMENT_H__
