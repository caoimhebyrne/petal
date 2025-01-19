#pragma once

#include "ast/node.h"
#include "core/type/type.h"

typedef struct {
    union {
        Node header;
    };

    // The name for this type.
    char* name;

    // The type being declared.
    Type* type;
} TypeDeclarationNode;

// Creates a new TypeDeclarationNode.
// Parameters:
// - position: The position that this node occurred at within the source file.
// - name: The name for this type.
// - type: The type being declared..
// Returns: A reference to an TypeDeclarationNode if successful, otherwise null.
TypeDeclarationNode* type_declaration_node_create(Position position, char* name, Type* type);

// Returns a string representation of an TypeDeclarationNode.
char* type_declaration_node_to_string(TypeDeclarationNode* node);

// De-allocates an TypeDeclarationNode's data.
// Parmaeters:
// - node: The TypeDeclarationNode to destroy.
void type_declaration_node_destroy(TypeDeclarationNode* node);
