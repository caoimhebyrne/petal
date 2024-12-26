#ifndef __BOOLEAN_LITERAL_H__
#define __BOOLEAN_LITERAL_H__

#include "../node.h"

typedef struct {
    // The type of this node, always NODE_BOOLEAN_LITERAL.
    NodeType node_type;

    // The position that this node occurred at within the source file.
    Position position;

    // The value that this node holds.
    bool value;
} BooleanLiteralNode;

// Creates a new BooleanLiteralNode.
// Parameters:
// - position: The position that this node occurred at within the source file.
// - value: The value that this node holds.
BooleanLiteralNode* boolean_literal_node_create(Position position, bool value);

// Returns a string representation of this boolean literal node.
char* boolean_literal_node_to_string(BooleanLiteralNode* node);

#endif // __BOOLEAN_LITERAL_H__
