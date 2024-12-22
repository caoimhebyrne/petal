#ifndef __FUNCTION_DECLARATION_NODE_H__
#define __FUNCTION_DECLARATION_NODE_H__

#include "../node.h"

typedef struct {
    // The type of this node, always NODE_FUNCTION_DECLARATION.
    NodeType node_type;

    // The name of this function.
    char* name;
} FunctionDeclarationNode;

// Creates a new function declaration node.
// Parameters:
// - name: The name of this function.
FunctionDeclarationNode* function_declaration_node_create(char* name);

// Returns a string representation of the provided FunctionDeclarationNode.
// Parameters:
// - node: The node to turn into a string.
// Returns:
// - A string representing the provided node.
char* function_declaration_node_to_string(FunctionDeclarationNode* node);

#endif // __FUNCTION_DECLARATION_NODE_H__
