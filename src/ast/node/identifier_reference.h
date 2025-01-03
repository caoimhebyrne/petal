#ifndef __AST_NODE_IDENTIFIER_REFERENCE_H__
#define __AST_NODE_IDENTIFIER_REFERENCE_H__

#include "ast/node.h"
#include "core/position.h"

typedef struct {
    union {
        Node header;
    };

    // The name of the identifier being referenced.
    char* identifier;
} IdentifierReferenceNode;

// Creates a new IdentifierReferenceNode.
// Parameters:
// - position: The position that this node occurred at within the source file.
// - identifier: The identifier being referenced.
// Returns: A reference to an IdentifierReferenceNode if successful, otherwise null.
IdentifierReferenceNode* identifier_reference_node_create(Position position, char* identifier);

// Returns a string representation of an IdentifierReferenceNode.
char* identifier_reference_node_to_string(IdentifierReferenceNode* node);

// De-allocates an IdentifierReferenceNode's data.
// Parmaeters:
// - node: The IdentifierReferenceNode to destroy.
void identifier_reference_node_destroy(IdentifierReferenceNode* node);

#endif // __AST_NODE_IDENTIFIER_REFERENCE_H__
