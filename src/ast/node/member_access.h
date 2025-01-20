#pragma once

#include "ast/node.h"
#include "core/position.h"

typedef struct {
    union {
        Node header;
    };

    // The node containing the member.
    Node* owner;

    // The member being accessed.
    char* member_name;

    // The index of the member being accessed, -1 if it is unknown.
    int member_index;
} MemberAccessNode;

// Creates a new MemberAccessNode
// Parameters:
// - position: The position that this node occurred at within the source file.
// - owner: The node containing the member.
// - member_name: The member being accessed.
MemberAccessNode* member_access_node_create(Position position, Node* owner, char* member_name);

// Returns a heap-allocated string representation of a MemberAccessNode.
char* member_access_node_to_string(MemberAccessNode* node);

// De-allocates an MemberAccessNode's data.
// Parmaeters:
// - node: The MemberAccessNode to destroy.
void member_access_node_destroy(MemberAccessNode* node);
