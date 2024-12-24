#include "function_call.h"
#include "../../string/format_string.h"

FunctionCallNode* function_call_node_create(Position position, char* name, NodeStream arguments) {
    FunctionCallNode* node = malloc(sizeof(FunctionCallNode));

    node->node_type = NODE_FUNCTION_CALL;
    node->position = position;
    node->name = name;
    node->arguments = arguments;

    return node;
}

char* function_call_node_to_string(FunctionCallNode* node) {
    return format_string("function call (name: '%s')", node->name);
}
