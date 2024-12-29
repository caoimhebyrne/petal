#ifndef __BINARY_OPERATION_H__
#define __BINARY_OPERATION_H__

#include "../node.h"
#include "../type.h"

typedef enum {
    OPERATOR_PLUS,
    OPERATOR_MINUS,
    OPERATOR_DIVIDE,
    OPERATOR_MULTIPLY,
} Operator;

typedef struct {
    // The type of this node, always NODE_BINARY_OPERATION.
    NodeType node_type;

    // The position that this node was parsed from the sourcefile.
    Position position;

    // The value on the left-hand side of this node.
    Node* left;

    // The value on the right-hand side of this node.
    Node* right;

    // The operator to perform on the two values.
    Operator operator_;

    // The expected type that this node should produce when the operation is performed.
    Type expected_type;
} BinaryOperationNode;

// Creates a binary operation node.
// Parameters:
// - position: The position that this node was parsed from the sourcefile.
// - left: The node on the left-hand side.
// - right: The node on the right-hand side.
// - operator: The operator being performed between the two values.
BinaryOperationNode* binary_operation_node_create(Position position, Node* left, Node* right, Operator operator_);

// Returns a string representation of the provided BinaryOperationNode.
// Parameters:
// - node: The node to turn into a string.
// Returns:
// - A string representing the provided node.
char* binary_operation_node_to_string(BinaryOperationNode* node);

// Returns a string representation of the provided Operator.
// Paramters:
// - operator: The operator to turn into a string.
// Returns:
// - A string representing the operator.
char* operator_to_string(Operator operator_);

#endif
