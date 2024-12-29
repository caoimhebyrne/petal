#ifndef __NUMBER_LITERAL_NODE_H__
#define __NUMBER_LITERAL_NODE_H__

#include "../node.h"
#include "../type.h"

typedef struct {
    // The type of this node, always NODE_NUMBER_LITERAL.
    NodeType node_type;

    // The rough position within the source file that this Node was
    // generated from.
    Position position;

    // The value.
    double value;

    // The expected type that this node produces.
    Type expected_type;
} NumberLiteralNode;

// Creates a new variable declaration node.
// Parameters:
// - value: The value of this node.
NumberLiteralNode* number_literal_node_create(Position position, double value);

// Returns a string representation of the provided VariableDeclarationNode.
// Parameters:
// - node: The node to turn into a string.
// Returns:
// - A string representing the provided node.
char* number_literal_node_to_string(NumberLiteralNode* node);

#endif // __NUMBER_LITERAL_NODE_H__
