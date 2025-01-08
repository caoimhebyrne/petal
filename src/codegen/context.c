#include "codegen/context.h"
#include "util/vector.h"
#include <string.h>

bool codegen_context_initialize(CodegenContext* context) {
    *context = (CodegenContext){.variables = vector_create()};
    return vector_initialize(context->variables, 1);
}

void codegen_context_destroy(CodegenContext* context) {
    if (context->variables.capacity == 0 || context->variables.items == nullptr) {
        return;
    }

    free(context->variables.items);
}

Variable* variable_find_by_name(VariableVector variables, char* name) {
    for (size_t i = 0; i < variables.length; i++) {
        auto variable = vector_get_ref(&variables, i);

        // FIXME: Use a hashtable here?
        if (strcmp(variable->name, name) == 0) {
            return variable;
        }
    }

    return nullptr;
}
