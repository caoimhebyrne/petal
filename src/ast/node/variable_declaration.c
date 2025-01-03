#include "variable_declaration.h"
#include "ast/node.h"
#include "util/format.h"
#include <stdlib.h>

VariableDeclarationNode* variable_declaration_node_create(Position position, char* type, char* name, Node* value) {
    VariableDeclarationNode* node = malloc(sizeof(VariableDeclarationNode));
    if (!node) {
        return nullptr;
    }

    node->header.kind = NODE_KIND_VARIABLE_DECLARATION;
    node->header.position = position;
    node->type = type;
    node->name = name;
    node->value = value;

    return node;
}

char* variable_declaration_node_to_string(VariableDeclarationNode* node) {
    auto value_string = node_to_string(node->value);
    auto string = format_string(
        "VariableDeclarationNode { name = '%s', type = '%s', value = '%s' }",
        node->name,
        node->type,
        value_string
    );

    free(value_string);
    return string;
}

void variable_declaration_node_destroy(VariableDeclarationNode* node) {
    free(node->type);
    free(node->name);

    node_destroy(node->value);
}
