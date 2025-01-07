#pragma once

#include "core/position.h"
#include "util/vector.h"

// Represents the different types of AST nodes.
typedef enum {
    // A binary operation node, see BinaryOperationNode.
    NODE_KIND_BINARY_OPERATION,

    // A function declaration node, see FunctionDeclarationNode.
    NODE_KIND_FUNCTION_DECLARATION,

    // An identifier reference node, see IdentifierReferenceNode.
    NODE_KIND_IDENTIFIER_REFERENCE,

    // A number literal node, see NumberLiteralNode.
    NODE_KIND_NUMBER_LITERAL,

    // A return node, see ReturnNode.
    NODE_KIND_RETURN,

    // A variable declaration node, see VariableDeclarationNode.
    NODE_KIND_VARIABLE_DECLARATION,
} NodeKind;

// The "base" for an AST node.
typedef struct {
    // The kind of Node that this is. The value of this member indicates the type that this `Node*` can be casted to.
    NodeKind kind;

    // The position that this node occurred at within the source file.
    Position position;
} Node;

// A vector of `Node*`.
typedef Vector(Node*) NodeVector;

// Returns a string representation of a Node.
// The caller should `free` this when it is no longer being used.
char* node_to_string(Node* node);

// De-allocates a single Node and all of its data.
// Parameters:
// - node: A reference to the node to destroy.
void node_destroy(Node* node);
