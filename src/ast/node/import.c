#include "import.h"
#include "../../string/format_string.h"
#include <stdlib.h>
#include <string.h>

ImportNode* import_node_create(Position position, char* module_name) {
    ImportNode* node = malloc(sizeof(ImportNode));
    if (!node) {
        return 0;
    }

    node->node_type = NODE_IMPORT;
    node->position = position;
    node->module_name = strdup(module_name);

    return node;
}

char* import_node_to_string(ImportNode* node) { return format_string("import module: '%s'", node->module_name); }
