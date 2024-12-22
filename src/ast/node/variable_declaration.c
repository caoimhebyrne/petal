#include "variable_declaration.h"
#include "../../string/format_string.h"
#include <stdlib.h>

VariableDeclarationNode* variable_declaration_node_create(char* type_name, char* name, Node* value) {
    VariableDeclarationNode* node = malloc(sizeof(VariableDeclarationNode));
    node->node_type = NODE_VARIABLE_DECLARATION;
    node->type_name = type_name;
    node->name = name;
    node->value = value;

    return node;
}

char* variable_declaration_node_to_string(VariableDeclarationNode* node) {
    return format_string("variable declaration (type: '%s', name: '%s', value: %s)", node->type_name, node->name,
                         node_to_string(node->value));
}
