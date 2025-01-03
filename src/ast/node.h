#ifndef __AST_NODE_H__
#define __AST_NODE_H__

#include "util/vector.h"

// Represents the different types of AST nodes.
typedef enum {
    // An identifier reference node, see IdentifierReferenceNode.
    NODE_KIND_IDENTIFIER_REFERENCE,
} NodeKind;

// The "base" for an AST node.
typedef struct {
    // The kind of Node that this is. The value of this member indicates the type that this `Node*` can be casted to.
    NodeKind kind;
} Node;

// A vector of `Node*`.
typedef Vector(Node*) NodeVector;

// De-allocates a single Node and all of its data.
// Parameters:
// - node: A reference to the node to destroy.
void node_destroy(Node* node);

#endif // __AST_NODE_H__
