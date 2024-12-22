#ifndef __FUNCTION_CALL_NODE_H__
#define __FUNCTION_CALL_NODE_H__

#include "../node.h"

// Represents a call to a function, which may include arguments.
typedef struct {
    // The type of this node, always NODE_FUNCTION_CALL.
    NodeType node_type;

    // The name of the function being called.
    char* name;
} FunctionCallNode;

// Creates a new function call node.
// Parameters:
// - name: The name of this function.
FunctionCallNode* function_call_node_create(char* name);

// Returns a string representation of the provided FunctionCallNode.
// Parameters:
// - node: The node to turn into a string.
// Returns:
// - A string representing the provided node.
char* function_call_node_to_string(FunctionCallNode* node);

#endif // __FUNCTION_CALL_NODE_H__