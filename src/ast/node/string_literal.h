#ifndef __STRING_LITERAL_NODE_H__
#define __STRING_LITERAL_NODE_H__

#include "../node.h"

typedef struct {
    // The type of this node, always NODE_STRING_LITERAL.
    NodeType node_type;

    // The rough position within the source file that this Node was
    // generated from.
    Position position;

    // The value.
    char* value;
} StringLiteralNode;

// Creates a new string literal node.
// Parameters:
// - value: The value of this node.
StringLiteralNode* string_literal_node_create(Position position, char* value);

// Returns a string representation of the provided StringLiteralNode.
// Parameters:
// - node: The node to turn into a string.
// Returns:
// - A string representing the provided node.
char* string_literal_node_to_string(StringLiteralNode* node);

#endif // __STRING_LITERAL_NODE_H__
