#pragma once

#include "ast/node.h"
#include "core/position.h"

typedef struct {
    // The name of the structure member.
    char* member_name;

    // The value being assigned to the member.
    Node* value;
} StructureMemberInitialization;

typedef Vector(StructureMemberInitialization) StructureMemberInitializationVector;

// Creates a new StructureMemberInitialization.
// Parameters:
// - member_name: The name of the structure member.
// - value: The value being assigned to the member.
StructureMemberInitialization structure_member_initialization_create(char* member_name, Node* value);

typedef struct {
    union {
        Node header;
    };

    // The members being initialized.
    StructureMemberInitializationVector members;
} StructureInitializationNode;

// Creates a new StructureInitializationNode with empty members.
// Parameters:
// - position: The position that this node occurred at within the source file.
StructureInitializationNode* structure_initialization_node_create(Position position);

// Returns a heap-allocated string representation of a StructureInitializationNode.
char* structure_initialization_node_to_string(StructureInitializationNode* node);

// Destroys a StructureInitializationNode.
void structure_initialization_node_destroy(StructureInitializationNode* node);
