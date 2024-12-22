#include "node.h"
#include "../string/format_string.h"
#include "node/function_declaration.h"
#include "node/identifier_reference.h"
#include "node/number_literal.h"
#include "node/variable_declaration.h"

CREATE_STREAM(NodeStream, node_stream, Node*);

void node_destroy(Node* node) {
    switch (node->node_type) {
    case NODE_VARIABLE_DECLARATION: {
        VariableDeclarationNode* variable_declaration = (VariableDeclarationNode*)node;
        free(variable_declaration->name);
        free(variable_declaration->type_name);
        node_destroy(variable_declaration->value);

        break;
    }

    case NODE_FUNCTION_DECLARATION: {
        FunctionDeclarationNode* function_declaration = (FunctionDeclarationNode*)node;
        free(function_declaration->name);
        node_stream_destroy(&function_declaration->function_body);

        break;
    }

    case NODE_NUMBER_LITERAL:
        break;

    case NODE_IDENTIFIER_REFERENCE: {
        IdentifierReferenceNode* identifier_reference = (IdentifierReferenceNode*)node;
        free(identifier_reference->name);

        break;
    }
    }
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

    case NODE_IDENTIFIER_REFERENCE:
        return identifier_reference_node_to_string((IdentifierReferenceNode*)node);
    }

    return format_string("unknown node (%d)", node->node_type);
}
