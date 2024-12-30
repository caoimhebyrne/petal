#ifndef __NODE_TYPE_ALIAS_DECLARATION_H__
#define __NODE_TYPE_ALIAS_DECLARATION_H__

#include "../node.h"
#include "../type.h"

typedef struct {
    // The type of this node, always NODE_TYPE_ALIAS_DECLARATION.
    NodeType node_type;

    // The position that this node occurred at within the source file.
    Position position;

    // The name being used for this type alias.
    char* name;

    // The type being aliased.
    Type* type;
} TypeAliasDeclarationNode;

// Creates a new type alias declaration node.
// Parameters:
// - position: The position that this node occurred at within the source file.
// - name: The name being used for this type alias.
// - type: The type being aliased.
TypeAliasDeclarationNode* type_alias_declaration_node_create(Position position, char* name, Type* type);

// Returns a string representation of a type alias declaration node.
char* type_alias_declaration_node_to_string(TypeAliasDeclarationNode* node);

#endif // __NODE_TYPE_ALIAS_DECLARATION_H__
