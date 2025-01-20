#include "ast/node/member_access.h"
#include "ast/node.h"
#include "core/position.h"
#include "util/defer.h"
#include "util/format.h"
#include <stdlib.h>

MemberAccessNode* member_access_node_create(Position position, Node* owner, char* member_name) {
    MemberAccessNode* node = malloc(sizeof(MemberAccessNode));
    if (!node) {
        return nullptr;
    }
    node->header.kind = NODE_KIND_MEMBER_ACCESS;
    node->header.position = position;
    node->owner = owner;
    node->member_name = member_name;

    return node;
}

char* member_access_node_to_string(MemberAccessNode* node) {
    auto node_string defer(free_str) = node_to_string(node->owner);
    return format_string("MemberAccessNode { owner = %s, member = '%s' }", node_string, node->member_name);
}

void member_access_node_destroy(MemberAccessNode* node) {
    free(node->member_name);
    node_destroy(node->owner);
}
