#ifndef __NODE_H__
#define __NODE_H__

#include "../position.h"
#include "../stream.h"

typedef enum {
    // A variable declaration node.
    // See VariableDeclarationNode.
    NODE_VARIABLE_DECLARATION,

    // A function declaration node.
    // See FunctionDeclarationNode.
    NODE_FUNCTION_DECLARATION,

    // A number literal node.
    // See NumberLiteralNode.
    NODE_NUMBER_LITERAL,

    // A string literal node.
    // See StringLiteralNode.
    NODE_STRING_LITERAL,

    // An identifier reference node.
    // See IdentifierReferenceNode.
    NODE_IDENTIFIER_REFERENCE,

    // A function call node.
    // See FunctionCallNode
    NODE_FUNCTION_CALL,

    // A return statement node.
    // See ReturnNode.
    NODE_RETURN,

    // A binary operation node.
    // See BinaryOperationNode.
    NODE_BINARY_OPERATION,
} NodeType;

typedef struct {
    // Represents the type of this node.
    // This value will indicate what type you can cast this Node to.
    NodeType node_type;

    // The rough position within the source file that this Node was
    // generated from.
    Position position;
} Node;

// Returns a string representation of the provided Node.
// Parameters:
// - node: The node to turn into a string.
// Returns:
// - A string representing the provided node.
char* node_to_string(Node* node);

DECLARE_STREAM(NodeStream, node_stream, Node*);

#endif // __NODE_H__
