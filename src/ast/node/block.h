#ifndef __BLOCK_NODE_H__
#define __BLOCK_NODE_H__

#include "../node.h"

typedef struct {
    // The type of this node, always NODE_BLOCK.
    NodeType node_type;

    // The position that this node occurred at within the source file.
    Position position;

    // The statements contained within this block.
    NodeStream body;
} BlockNode;

// Creates a new block node.
// Parameters:
// - position: The position that this node occurred at within the source file.
// - body: The statements contained within this block.
BlockNode* block_node_create(Position position, NodeStream body);

// Returns a string representation of a block node.
char* block_node_to_string(BlockNode* node);

#endif // __BLOCK_NODE__H
