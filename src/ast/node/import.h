#ifndef __NODE_IMPORT_H__
#define __NODE_IMPORT_H__

#include "../node.h"

typedef struct {
    // The type of this node, always NODE_IMPORT.
    NodeType node_type;

    // The position that this node occurred at within the source file.
    Position position;

    // The name of the module being imported.
    char* module_name;
} ImportNode;

// Creates a new import node.
// Parameters:
// - position: The position that this node occurred at within the source file.
// - module_name: The name of the module being imported.
ImportNode* import_node_create(Position position, char* module_name);

// Returns a string representation of the provided import node.
char* import_node_to_string(ImportNode* node);

#endif // __NODE_IMPORT_H__
