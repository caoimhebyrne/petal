#include "node.h"
#include "../string/format_string.h"
#include "node/binary_operation.h"
#include "node/block.h"
#include "node/boolean_literal.h"
#include "node/force_unwrap.h"
#include "node/function_call.h"
#include "node/function_declaration.h"
#include "node/identifier_reference.h"
#include "node/number_literal.h"
#include "node/return.h"
#include "node/string_literal.h"
#include "node/type_alias_declaration.h"
#include "node/variable_declaration.h"
#include "node/variable_reassignment.h"
#include "type.h"

CREATE_STREAM(NodeStream, node_stream, Node*);

void node_destroy(Node* node) {
    switch (node->node_type) {
    case NODE_VARIABLE_DECLARATION: {
        VariableDeclarationNode* variable_declaration = (VariableDeclarationNode*)node;
        free(variable_declaration->name);
        node_destroy(variable_declaration->value);
        type_destroy(variable_declaration->type);

        break;
    }

    case NODE_FUNCTION_DECLARATION: {
        FunctionDeclarationNode* function_declaration = (FunctionDeclarationNode*)node;
        free(function_declaration->name);

        if (function_declaration->function_body) {
            node_destroy((Node*)function_declaration->function_body);
        }

        type_destroy(function_declaration->return_type);

        break;
    }

    case NODE_BOOLEAN_LITERAL:
        break;

    case NODE_NUMBER_LITERAL: {
        NumberLiteralNode* number_literal = (NumberLiteralNode*)node;
        type_destroy(number_literal->type);

        break;
    }

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
        type_destroy(binary_operation->type);

        node_destroy(binary_operation->left);
        node_destroy(binary_operation->right);

        break;
    }

    case NODE_BLOCK: {
        BlockNode* block = (BlockNode*)node;
        node_stream_destroy(&block->body);

        break;
    }

    case NODE_TYPE_ALIAS_DECLARATION: {
        TypeAliasDeclarationNode* type_alias_declaration = (TypeAliasDeclarationNode*)node;
        free(type_alias_declaration->name);
        type_destroy(type_alias_declaration->type);

        break;
    }

    case NODE_VARIABLE_REASSIGNMENT: {
        VariableReassignmentNode* variable_reassignment = (VariableReassignmentNode*)node;

        node_destroy(variable_reassignment->value);
        free(variable_reassignment->variable_name);

        break;
    }

    case NODE_FORCE_UNWRAP: {
        ForceUnwrapNode* force_unwrap = (ForceUnwrapNode*)node;
        node_destroy(force_unwrap->value);

        break;
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

    case NODE_BOOLEAN_LITERAL:
        return boolean_literal_node_to_string((BooleanLiteralNode*)node);

    case NODE_TYPE_ALIAS_DECLARATION:
        return type_alias_declaration_node_to_string((TypeAliasDeclarationNode*)node);

    case NODE_VARIABLE_REASSIGNMENT:
        return variable_reassignment_node_to_string((VariableReassignmentNode*)node);

    case NODE_FORCE_UNWRAP:
        return force_unwrap_node_to_string((ForceUnwrapNode*)node);
    }

    return format_string("unknown node (%d)", node->node_type);
}
