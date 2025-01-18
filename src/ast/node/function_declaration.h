#pragma once

#include "ast/node.h"
#include "core/parameter.h"
#include "core/type/type.h"

typedef enum {
    FUNCTION_MODIFIER_EXTERN = (1u << 0),
} FunctionModifier;

typedef struct {
    union {
        Node header;
    };

    // The name of this function.
    char* name;

    // The return type of this function.
    Type* return_type;

    // The parameters that this function expects.
    ParameterVector parameters;

    // The function's body.
    NodeVector body;

    int32_t modifiers;
} FunctionDeclarationNode;

// Creates a new FunctionDeclarationNode.
// Parameters:
// - position: The position that this node occurred at within the source file.
// - name: The name of this function.
// - return_type: The return type of this function.
// - parameters: The parameters that this function expects.
// - body: The function's body.
// Returns: A reference to an FunctionDeclarationNode if successful, otherwise null.
FunctionDeclarationNode* function_declaration_node_create(
    Position position,
    char* name,
    Type* return_type,
    ParameterVector parameters,
    NodeVector body,
    int32_t modifiers
);

// Returns a string representation of an FunctionDeclarationNode.
char* function_declaration_node_to_string(FunctionDeclarationNode* node);

// De-allocates an FunctionDeclarationNode's data.
// Parmaeters:
// - node: The FunctionDeclarationNode to destroy.
void function_declaration_node_destroy(FunctionDeclarationNode* node);
