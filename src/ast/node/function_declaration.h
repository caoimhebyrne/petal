#ifndef __FUNCTION_DECLARATION_NODE_H__
#define __FUNCTION_DECLARATION_NODE_H__

#include "../node.h"
#include "../type.h"

typedef struct {
    // The type of this node, always NODE_FUNCTION_DECLARATION.
    NodeType node_type;

    // The rough position within the source file that this Node was
    // generated from.
    Position position;

    // The name of this function.
    char* name;

    // The return type of this function
    Type return_type;

    // The nodes contained within this function's body.
    NodeStream function_body;
} FunctionDeclarationNode;

// Creates a new function declaration node.
// Parameters:
// - name: The name of this function.
// - return_type: The return type of this function.
// - node_stream: The nodes contained within this function's body.
FunctionDeclarationNode* function_declaration_node_create(Position position, char* name, Type return_type,
                                                          NodeStream function_body);

// Returns a string representation of the provided FunctionDeclarationNode.
// Parameters:
// - node: The node to turn into a string.
// Returns:
// - A string representing the provided node.
char* function_declaration_node_to_string(FunctionDeclarationNode* node);

#endif // __FUNCTION_DECLARATION_NODE_H__
