#ifndef __IDENTIFIER_REFERENCE_NODE_H__
#define __IDENTIFIER_REFERENCE_NODE_H__

#include "../node.h"

// A node which is emitted when an identifier is used as a "value reference".
typedef struct {
    // The type of this node, always NODE_IDENTIFIER_REFERENCE.
    NodeType node_type;

    // The rough position within the source file that this Node was
    // generated from.
    Position position;

    // The name of the identifier referenced.
    char* name;

    // Whether this should be passed by reference or not.
    bool by_reference;
} IdentifierReferenceNode;

// Creates a new IdentifierReferenceNode.
// Parameters:
// - name: The name of the identifier referenced.
// - by_reference: Whether this should be passed by reference or not.
IdentifierReferenceNode* identifier_reference_node_create(Position position, char* name, bool by_reference);

// Returns a string representation of the provided IdentifierReferenceNode.
// Parameters:
// - node: The node to turn into a string.
// Returns:
// - A string representing the provided node.
char* identifier_reference_node_to_string(IdentifierReferenceNode* node);

#endif // __IDENTIFIER_REFERENCE_NODE_H__
