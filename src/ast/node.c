#include "ast/node.h"
#include "ast/node/binary_operation.h"
#include "ast/node/function_call.h"
#include "ast/node/function_declaration.h"
#include "ast/node/identifier_reference.h"
#include "ast/node/number_literal.h"
#include "ast/node/return.h"
#include "ast/node/variable_declaration.h"
#include "ast/node/variable_reassignment.h"

char* node_to_string(Node* node) {
    switch (node->kind) {
    case NODE_KIND_BINARY_OPERATION:
        return binary_operation_node_to_string((BinaryOperationNode*)node);

    case NODE_KIND_FUNCTION_DECLARATION:
        return function_declaration_node_to_string((FunctionDeclarationNode*)node);

    case NODE_KIND_IDENTIFIER_REFERENCE:
        return identifier_reference_node_to_string((IdentifierReferenceNode*)node);

    case NODE_KIND_NUMBER_LITERAL:
        return number_literal_node_to_string((NumberLiteralNode*)node);

    case NODE_KIND_RETURN:
        return return_node_to_string((ReturnNode*)node);

    case NODE_KIND_VARIABLE_DECLARATION:
        return variable_declaration_node_to_string((VariableDeclarationNode*)node);

    case NODE_KIND_FUNCTION_CALL:
        return function_call_node_to_string((FunctionCallNode*)node);

    case NODE_KIND_VARIABLE_REASSIGNMENT:
        return variable_reassignment_node_to_string((VariableReassignmentNode*)node);
    }
}

void node_destroy(Node* node) {
    switch (node->kind) {
    case NODE_KIND_BINARY_OPERATION:
        binary_operation_node_destroy((BinaryOperationNode*)node);
        break;

    case NODE_KIND_FUNCTION_DECLARATION:
        function_declaration_node_destroy((FunctionDeclarationNode*)node);
        break;

    case NODE_KIND_IDENTIFIER_REFERENCE:
        identifier_reference_node_destroy((IdentifierReferenceNode*)node);
        break;

    case NODE_KIND_NUMBER_LITERAL:
        number_literal_node_destroy((NumberLiteralNode*)node);
        break;

    case NODE_KIND_RETURN:
        return_node_destroy((ReturnNode*)node);
        break;

    case NODE_KIND_VARIABLE_DECLARATION:
        variable_declaration_node_destroy((VariableDeclarationNode*)node);
        break;

    case NODE_KIND_FUNCTION_CALL:
        function_call_node_destroy((FunctionCallNode*)node);
        break;

    case NODE_KIND_VARIABLE_REASSIGNMENT:
        variable_reassignment_node_destroy((VariableReassignmentNode*)node);
        break;
    }

    free(node);
}
