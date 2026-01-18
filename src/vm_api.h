#pragma once

#include "vm_value.h"
#include <stddef.h>

/**
 * An opaque pointer to the arguments passed during a function call.
 */
typedef struct PetalBuiltinArguments PetalBuiltinArguments;

/**
 * Returns the number of arguments in a PetalBuiltinContext.
 */
size_t petal_builtin_arguments_length(const PetalBuiltinArguments* arguments);

/**
 * Returns a pointer to the [VMValue] corresponding to the argument at the provided index.
 * If the provided index is out of bounds, a NULL pointer is returned.
 */
const VMValue* petal_builtin_arguments_get(const PetalBuiltinArguments* arguments, const size_t index);

/**
 * The context passed to a built-in function handler.
 */
typedef struct {
    /**
     * The arguments to this function call.
     */
    PetalBuiltinArguments* arguments;
} PetalBuiltinContext;
