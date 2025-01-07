#pragma once

#include "ast/node.h"

typedef struct {
    union {
        Node header;
    };

    // The value being returned, can be `nullptr`.
    Node* return_value;
} ReturnNode;

// Creates a new ReturnNode.
// Parameters:
// - position: The position that this node occurred at within the source file.
// - return_value: The value being returned, can be `nullptr`.
// Returns: A reference to an ReturnNode if successful, otherwise null.
ReturnNode* return_node_create(Position position, Node* return_value);

// Returns a string representation of an ReturnNode.
char* return_node_to_string(ReturnNode* node);

// De-allocates an ReturnNode's data.
// Parmaeters:
// - node: The ReturnNode to destroy.
void return_node_destroy(ReturnNode* node);
