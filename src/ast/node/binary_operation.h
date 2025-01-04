#pragma once

#include "ast/node.h"

typedef enum {
    OPERATOR_ADD,
    OPERATOR_SUBTRACT,
    OPERATOR_MULTIPLY,
    OPERATOR_DIVIDE,
} Operator;

// Returns a string representation of an Operator.
const char* operator_to_string(Operator operator);

typedef struct {
    union {
        Node header;
    };

    // The left-hand side of the node.
    Node* left;

    // The operation being performed between the left and right hand sides.
    Operator operator;

    // The right-hand side of the node.
    Node* right;
} BinaryOperationNode;

// Creates a new BinaryOperationNode.
// Parameters:
// - position: The position that this node occurred at within the source file.
// - left: The left-hand side of the node.
// - operator: The operation being performed between the left and right hand sides.
// - right: The right-hand side of the node.
// Returns: A reference to an BinaryOperationNode if successful, otherwise null.
BinaryOperationNode* binary_operation_node_create(Position position, Node* left, Operator operator, Node * right);

// Returns a string representation of an BinaryOperationNode.
char* binary_operation_node_to_string(BinaryOperationNode* node);

// De-allocates an BinaryOperationNode's data.
// Parmaeters:
// - node: The BinaryOperationNode to destroy.
void binary_operation_node_destroy(BinaryOperationNode* node);
