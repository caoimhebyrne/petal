#include "ast/node/structure_initialization.h"
#include "util/format.h"
#include "util/vector.h"
#include <stdlib.h>

StructureMemberInitialization structure_member_initialization_create(char* member_name, Node* value) {
    return (StructureMemberInitialization){member_name, value};
}

void structure_member_initialization_destroy(StructureMemberInitialization member) {
    free(member.member_name);
    node_destroy(member.value);
}

StructureInitializationNode* structure_initialization_node_create(Position position) {
    StructureInitializationNode* node = malloc(sizeof(StructureInitializationNode));
    if (!node) {
        return nullptr;
    }

    node->header.kind = NODE_KIND_STRUCTURE_INITIALIZATION;
    node->header.position = position;
    node->members = (StructureMemberInitializationVector){};

    if (!vector_initialize(node->members, 1)) {
        return nullptr;
    }

    return node;
}

char* structure_initialization_node_to_string(StructureInitializationNode* node) {
    return format_string("StructureInitializationNode { members.length: %d }", node->members.length);
}

void structure_initialization_node_destroy(StructureInitializationNode* node) {
    vector_destroy(node->members, structure_member_initialization_destroy);
}
