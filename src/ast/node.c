#include "node.h"
#include "../string/format_string.h"
#include "node/function_declaration.h"
#include "node/number_literal.h"
#include "node/variable_declaration.h"

CREATE_STREAM(NodeStream, node_stream, Node*);

void node_stream_destroy(NodeStream* stream) { free(stream->data); }

char* node_to_string(Node* node) {
    switch (node->node_type) {
    case NODE_VARIABLE_DECLARATION:
        return variable_declaration_node_to_string((VariableDeclarationNode*)node);

    case NODE_FUNCTION_DECLARATION:
        return function_declaration_node_to_string((FunctionDeclarationNode*)node);

    case NODE_NUMBER_LITERAL:
        return number_literal_node_to_string((NumberLiteralNode*)node);
    }

    return format_string("unknown node (%d)", node->node_type);
}
