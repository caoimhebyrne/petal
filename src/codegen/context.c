#include "codegen/context.h"
#include "util/vector.h"

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
