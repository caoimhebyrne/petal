#include "vm_api.h"
#include "allocator.h"
#include "vm_api_internal.h"
#include "vm_value.h"
#include <assert.h>

void petal_builtin_context_init(PetalBuiltinContext* context, Allocator* allocator, const VMValueArray* arguments) {
    PetalBuiltinArguments* builtin_arguments = allocator_alloc(allocator, sizeof(PetalBuiltinArguments));
    assert(builtin_arguments != NULL && "Failed to allocate PetalBuiltinArguments!");
    builtin_arguments->values = arguments;

    context->arguments = builtin_arguments;
}

size_t petal_builtin_arguments_length(const PetalBuiltinArguments* arguments) {
    return arguments->values->length;
}

const VMValue* petal_builtin_arguments_get(const PetalBuiltinArguments* arguments, const size_t index) {
    if (index >= arguments->values->length) {
        return NULL;
    }

    return &arguments->values->data[index];
}
