#pragma once

#include "ast/node.h"
#include "core/position.h"

typedef struct {
    union {
        Node header;
    };

    // The name of the function being called.
    char* function_name;

    // The arguments being passed to the function.
    NodeVector arguments;
} FunctionCallNode;

// Creates a new FunctionCallNode.
// Parameters:
// - position: The position that this node occurred at within the source file.
// - function_name: The name of the function being called.
// - arguments: The arguments being passed to the function.
// Returns: A reference to an FunctionCallNode if successful, otherwise null.
FunctionCallNode* function_call_node_create(Position position, char* function_name, NodeVector arguments);

// Returns a string representation of an FunctionCallNode.
char* function_call_node_to_string(FunctionCallNode* node);

// De-allocates an FunctionCallNode's data.
// Parmaeters:
// - node: The FunctionCallNode to destroy.
void function_call_node_destroy(FunctionCallNode* node);
