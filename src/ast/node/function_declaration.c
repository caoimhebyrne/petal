#include "function_declaration.h"
#include "../../string/format_string.h"
#include <stdlib.h>

FunctionDeclarationNode* function_declaration_node_create(char* name, Type return_type, NodeStream function_body) {
    FunctionDeclarationNode* node = malloc(sizeof(FunctionDeclarationNode));
    node->node_type = NODE_FUNCTION_DECLARATION;
    node->name = name;
    node->return_type = return_type;
    node->function_body = function_body;

    return node;
}

char* function_declaration_node_to_string(FunctionDeclarationNode* node) {
    return format_string("function declaration (name: '%s', return type: '%s')", node->name,
                         type_to_string(node->return_type));
}
