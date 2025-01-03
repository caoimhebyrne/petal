#pragma once

#include "ast/node.h"
#include "core/position.h"
#include "core/type.h"

typedef struct {
    union {
        Node header;
    };

    // The type of this variable.
    Type* type;

    // The name of this variable.
    char* name;

    // The value being assigned to this variable.
    // FIXME: Make this nullable?
    Node* value;
} VariableDeclarationNode;

// Creates a new VariableDeclarationNode.
// Parameters:
// - position: The position that this node occurred at within the source file.
// - type: The type of the variable.
// - name: The name of the variable.
// - value: The value being assigned to the variable.
VariableDeclarationNode* variable_declaration_node_create(Position position, Type* type, char* name, Node* value);

// Returns a string representation of a VariableDeclarationNode.
char* variable_declaration_node_to_string(VariableDeclarationNode* node);

// De-allocates an VariableDeclarationNode's data.
// Parmaeters:
// - node: The VariableDeclarationNode to destroy.
void variable_declaration_node_destroy(VariableDeclarationNode* node);
