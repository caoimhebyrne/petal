#include "ast/node.h"
#include "ast/node/binary_operation.h"
#include "ast/node/identifier_reference.h"
#include "ast/node/variable_declaration.h"

char* node_to_string(Node* node) {
    switch (node->kind) {
    case NODE_KIND_BINARY_OPERATION:
        return binary_operation_node_to_string((BinaryOperationNode*)node);

    case NODE_KIND_IDENTIFIER_REFERENCE:
        return identifier_reference_node_to_string((IdentifierReferenceNode*)node);

    case NODE_KIND_VARIABLE_DECLARATION:
        return variable_declaration_node_to_string((VariableDeclarationNode*)node);
    }
}

void node_destroy(Node* node) {
    switch (node->kind) {
    case NODE_KIND_BINARY_OPERATION:
        binary_operation_node_destroy((BinaryOperationNode*)node);
        break;

    case NODE_KIND_IDENTIFIER_REFERENCE:
        identifier_reference_node_destroy((IdentifierReferenceNode*)node);
        break;

    case NODE_KIND_VARIABLE_DECLARATION:
        variable_declaration_node_destroy((VariableDeclarationNode*)node);
        break;
    }

    free(node);
}
