#ifndef __NODE_FORCE_UNWRAP_H__
#define __NODE_FORCE_UNWRAP_H__

#include "../node.h"

typedef struct {
    // The type of this node, always NODE_FORCE_UNWRAP.
    NodeType node_type;

    // The position that this node occurred at within the source file.
    Position position;

    // The value being forcefully unwrapped.
    Node* value;
} ForceUnwrapNode;

// Creates a new force-unwrap node.
// Parameters:
// - position: The position that this node occurred at within the source file.
// - value: The value being forcefully unwrapped.
ForceUnwrapNode* force_unwrap_node_create(Position position, Node* value);

// Returns a string representation of a force-unwrap node.
char* force_unwrap_node_to_string(ForceUnwrapNode* node);

#endif // __NODE_FORCE_UNWRAP_H__
