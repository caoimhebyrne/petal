#include "variable_declaration.h"
#include "../../string/format_string.h"
#include <stdlib.h>

VariableDeclarationNode* variable_declaration_node_create(char* name, Type type, Node* value) {
    VariableDeclarationNode* node = malloc(sizeof(VariableDeclarationNode));
    node->node_type = NODE_VARIABLE_DECLARATION;
    node->name = name;
    node->type = type;
    node->value = value;

    return node;
}

char* variable_declaration_node_to_string(VariableDeclarationNode* node) {
    return format_string("variable declaration (type: '%s', name: '%s', value: %s)", type_to_string(node->type),
                         node->name, node_to_string(node->value));
}
