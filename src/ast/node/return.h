#ifndef __RETURN_STATEMENT_NODE_H__
#define __RETURN_STATEMENT_NODE_H__

#include "../node.h"

typedef struct {
    // The type of this node, always NODE_RETURN.
    NodeType node_type;

    // The value being returned.
    // This may be null! Be careful!
    Node* value;
} ReturnNode;

// Creates a return statement node.
// Parameters:
// - value: The value being returned
ReturnNode* return_node_create(Node* value);

// Returns a string representation of the provided return statement node.
// Parameters:
// - node: The node to return a string representation of.
char* return_node_to_string(ReturnNode* node);

#endif // __RETURN_STATEMENT_NODE_H__
