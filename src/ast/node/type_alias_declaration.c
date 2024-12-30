#include "type_alias_declaration.h"
#include "../../string/format_string.h"
#include <stdlib.h>

TypeAliasDeclarationNode* type_alias_declaration_node_create(Position position, char* name, Type* type) {
    TypeAliasDeclarationNode* node = malloc(sizeof(TypeAliasDeclarationNode));
    if (!node) {
        return 0;
    }

    node->node_type = NODE_TYPE_ALIAS_DECLARATION;
    node->position = position;
    node->name = name;
    node->type = type;

    return node;
}

char* type_alias_declaration_node_to_string(TypeAliasDeclarationNode* node) {
    return format_string("type alias declaration (name: '%s', type: '%s')", node->name, type_to_string(node->type));
}
