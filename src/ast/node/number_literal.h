#pragma once

#include "ast/node.h"
#include "core/type/type.h"
#include <stdint.h>

typedef struct {
    union {
        Node header;
    };

    // Whether this number literal is a float.
    // If true, use `number`, otherwise use `integer`.
    // FIXME: Replace with a `Type` in the future?
    bool is_float;

    // The expected type for this number literal.
    Type* type;

    union {
        // Only available if `is_float` is true.
        double number;

        // Only available if `is_float` is false.
        uint64_t integer;
    };
} NumberLiteralNode;

// Creates a new NumberLiteralNode with a floating point value.
// Parameters:
// - position: The position that this node occurred at within the source file.
// - value: The floating point value.
// Returns: A reference to an NumberLiteralNode if successful, otherwise null.
NumberLiteralNode* number_literal_node_create_float(Position position, double value);

// Creates a new NumberLiteralNode with a whole integer value.
// Parameters:
// - position: The position that this node occurred at within the source file.
// - value: The integer value.
// Returns: A reference to an NumberLiteralNode if successful, otherwise null.
NumberLiteralNode* number_literal_node_create_integer(Position position, uint64_t value);

// Returns a string representation of an NumberLiteralNode.
char* number_literal_node_to_string(NumberLiteralNode* node);

// De-allocates a NumberLiteralNode's data.
// Parmaeters:
// - node: The NumberLiteralNode to destroy.
void number_literal_node_destroy(NumberLiteralNode* node);
