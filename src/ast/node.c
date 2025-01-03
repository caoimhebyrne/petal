#include "ast/node.h"

void node_destroy(Node* node) {
    // FIXME: Call destroy functions based on the NodeKind.
    free(node);
}
