#include "function_declaration.h"
#include "../../string/format_string.h"
#include <stdlib.h>

FunctionDeclarationNode* function_declaration_node_create(Position position, char* name, Parameters parameters,
                                                          Type return_type, NodeStream function_body,
                                                          bool is_external) {
    FunctionDeclarationNode* node = malloc(sizeof(FunctionDeclarationNode));
    node->node_type = NODE_FUNCTION_DECLARATION;
    node->position = position;
    node->name = name;
    node->parameters = parameters;
    node->return_type = return_type;
    node->function_body = function_body;
    node->is_external = is_external;

    return node;
}

char* function_declaration_node_to_string(FunctionDeclarationNode* node) {
    return format_string("function declaration (name: '%s', return type: '%s', is external: %d)", node->name,
                         type_to_string(node->return_type), node->is_external);
}
