#pragma once

#include "allocator.h"
#include "vm_api.h"
#include "vm_value.h"

struct PetalBuiltinArguments {
    /**
     * The raw `VMValue` instances.
     */
    const VMValueArray* values;
};

/**
 * Initializes a [PetalBuiltinContext] with the provided data.
 */
void petal_builtin_context_init(PetalBuiltinContext* context, Allocator* allocator, const VMValueArray* arguments);
