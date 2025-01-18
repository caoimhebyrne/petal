#include "function_call.h"
#include "ast/node.h"
#include "util/format.h"
#include <stdlib.h>

FunctionCallNode* function_call_node_create(Position position, char* function_name, NodeVector arguments) {
    FunctionCallNode* node = malloc(sizeof(FunctionCallNode));
    if (!node) {
        return nullptr;
    }

    node->header.kind = NODE_KIND_FUNCTION_CALL;
    node->header.position = position;
    node->function_name = function_name;
    node->arguments = arguments;

    return node;
}

char* function_call_node_to_string(FunctionCallNode* node) {
    return format_string("FunctionCallNode { identifier = '%s' }", node->function_name);
}

void function_call_node_destroy(FunctionCallNode* node) {
    free(node->function_name);
    vector_destroy(node->arguments, node_destroy);
}
