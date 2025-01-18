#pragma once

#include "ast/node.h"
#include "core/position.h"
#include "core/type/type.h"

typedef struct {
    union {
        Node header;
    };

    // The name of the identifier being referenced.
    char* identifier;

    // The type associated with this identifier reference node.
    // FIXME: This is only here to prevent a memory leak with references lol
    Type* value_type;
} IdentifierReferenceNode;

// Creates a new IdentifierReferenceNode.
// Parameters:
// - position: The position that this node occurred at within the source file.
// - identifier: The identifier being referenced.
// - value_type: The type associated with this identifier reference node.
// Returns: A reference to an IdentifierReferenceNode if successful, otherwise null.
IdentifierReferenceNode* identifier_reference_node_create(Position position, char* identifier, Type* value_type);

// Returns a string representation of an IdentifierReferenceNode.
char* identifier_reference_node_to_string(IdentifierReferenceNode* node);

// De-allocates an IdentifierReferenceNode's data.
// Parmaeters:
// - node: The IdentifierReferenceNode to destroy.
void identifier_reference_node_destroy(IdentifierReferenceNode* node);
