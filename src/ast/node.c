#include "node.h"
#include "../string/format_string.h"
#include "node/binary_operation.h"
#include "node/block.h"
#include "node/function_call.h"
#include "node/function_declaration.h"
#include "node/identifier_reference.h"
#include "node/number_literal.h"
#include "node/return.h"
#include "node/string_literal.h"
#include "node/variable_declaration.h"

CREATE_STREAM(NodeStream, node_stream, Node*);

void node_destroy(Node* node) {
    switch (node->node_type) {
    case NODE_VARIABLE_DECLARATION: {
        VariableDeclarationNode* variable_declaration = (VariableDeclarationNode*)node;
        free(variable_declaration->name);
        node_destroy(variable_declaration->value);

        break;
    }

    case NODE_FUNCTION_DECLARATION: {
        FunctionDeclarationNode* function_declaration = (FunctionDeclarationNode*)node;
        free(function_declaration->name);

        if (function_declaration->function_body) {
            node_destroy((Node*)function_declaration->function_body);
        }

        break;
    }

    case NODE_NUMBER_LITERAL:
        break;

    case NODE_STRING_LITERAL: {
        StringLiteralNode* string_literal = (StringLiteralNode*)node;
        free(string_literal->value);

        break;
    }

    case NODE_IDENTIFIER_REFERENCE: {
        IdentifierReferenceNode* identifier_reference = (IdentifierReferenceNode*)node;
        free(identifier_reference->name);

        break;
    }

    case NODE_FUNCTION_CALL: {
        FunctionCallNode* function_call = (FunctionCallNode*)node;
        free(function_call->name);

        break;
    }

    case NODE_RETURN: {
        ReturnNode* return_statement = (ReturnNode*)node;
        if (return_statement->value != 0) {
            node_destroy(return_statement->value);
        }

        break;
    }

    case NODE_BINARY_OPERATION: {
        BinaryOperationNode* binary_operation = (BinaryOperationNode*)node;
        node_destroy(binary_operation->left);
        node_destroy(binary_operation->right);

        break;
    }

    case NODE_BLOCK: {
        BlockNode* block = (BlockNode*)node;
        node_stream_destroy(&block->body);
    }
    }

    free(node);
}

void node_stream_destroy(NodeStream* stream) {
    for (size_t i = 0; i < stream->length; i++) {
        node_destroy(stream->data[i]);
    }

    free(stream->data);
}

char* node_to_string(Node* node) {
    switch (node->node_type) {
    case NODE_VARIABLE_DECLARATION:
        return variable_declaration_node_to_string((VariableDeclarationNode*)node);

    case NODE_FUNCTION_DECLARATION:
        return function_declaration_node_to_string((FunctionDeclarationNode*)node);

    case NODE_NUMBER_LITERAL:
        return number_literal_node_to_string((NumberLiteralNode*)node);

    case NODE_STRING_LITERAL:
        return string_literal_node_to_string((StringLiteralNode*)node);

    case NODE_IDENTIFIER_REFERENCE:
        return identifier_reference_node_to_string((IdentifierReferenceNode*)node);

    case NODE_FUNCTION_CALL:
        return function_call_node_to_string((FunctionCallNode*)node);

    case NODE_RETURN:
        return return_node_to_string((ReturnNode*)node);

    case NODE_BINARY_OPERATION:
        return binary_operation_node_to_string((BinaryOperationNode*)node);

    case NODE_BLOCK:
        return block_node_to_string((BlockNode*)node);
    }

    return format_string("unknown node (%d)", node->node_type);
}
