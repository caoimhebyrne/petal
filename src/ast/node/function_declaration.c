#include "function_declaration.h"
#include "../../string/format_string.h"
#include <stdlib.h>

FunctionDeclarationNode* function_declaration_node_create(char* name) {
    FunctionDeclarationNode* node = malloc(sizeof(FunctionDeclarationNode));
    node->node_type = NODE_FUNCTION_DECLARATION;
    node->name = name;

    return node;
}

char* function_declaration_node_to_string(FunctionDeclarationNode* node) {
    return format_string("function declaration (name: '%s')", node->name);
}
