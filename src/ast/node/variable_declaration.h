#ifndef __VARIABLE_DECLARATION_NODE_H__
#define __VARIABLE_DECLARATION_NODE_H__

#include "../node.h"
#include "../type.h"

typedef struct {
    // The type of this node, always NODE_VARIABLE_DECLARATION.
    NodeType node_type;

    // The rough position within the source file that this Node was
    // generated from.
    Position position;

    // The name of this variable.
    char* name;

    // The type of this variable.
    Type type;

    // The value being assigned to this variable.
    Node* value;
} VariableDeclarationNode;

// Creates a new variable declaration node, given a type name, name, and value node.
// Parameters:
// - name: The name of this variable.
// - type: The type of this variable.
// - value: The value being assigned to this variable.
VariableDeclarationNode* variable_declaration_node_create(Position position, char* name, Type type, Node* value);

// Returns a string representation of the provided VariableDeclarationNode.
// Parameters:
// - node: The node to turn into a string.
// Returns:
// - A string representing the provided node.
char* variable_declaration_node_to_string(VariableDeclarationNode* node);

#endif // __VARIABLE_DECLARATION_NODE_H__
