#include "block.h"
#include "../../string/format_string.h"
#include <stdlib.h>

BlockNode* block_node_create(Position position, NodeStream body) {
    BlockNode* node = malloc(sizeof(BlockNode));
    if (!node) {
        return 0;
    }

    node->node_type = NODE_BLOCK;
    node->position = position;
    node->body = body;

    return node;
}

char* block_node_to_string(BlockNode* node) { return format_string("block node (%zu statements)", node->body.length); }
