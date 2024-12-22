#ifndef __VARIABLE_DECLARATION_NODE_H__
#define __VARIABLE_DECLARATION_NODE_H__

#include "../node.h"

typedef struct {
    // The type of this node, always NODE_VARIABLE_DECLARATION.
    NodeType node_type;

    // The name of the type of this variable.
    char* type_name;

    // The name of this variable.
    char* name;

    // The value being assigned to this variable.
    Node* value;
} VariableDeclarationNode;

// Creates a new variable declaration node, given a type name, name, and value node.
// Parameters:
// - type_name: The name of the type of this variable.
// - name: The name of this variable.
// - value: The value being assigned to this variable.
VariableDeclarationNode* variable_declaration_node_create(char* type_name, char* name, Node* value);

// Returns a string representation of the provided VariableDeclarationNode.
// Parameters:
// - node: The node to turn into a string.
// Returns:
// - A string representing the provided node.
char* variable_declaration_node_to_string(VariableDeclarationNode* node);

#endif // __VARIABLE_DECLARATION_NODE_H__
