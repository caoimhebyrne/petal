#pragma once

#include "vm_api.h"

/**
 * The type of a built-in function handler.
 */
typedef VMValue (*VMBuiltinFunctionHandler)(const PetalBuiltinContext* context);

/**
 * A built-in function.
 */
typedef struct {
    // The name of the built-in function.
    const char* name;

    // The C function to call to handle the function.
    VMBuiltinFunctionHandler handler;
} VMBuiltinFunction;

/**
 * A list of all configured VM builtins.
 */
extern VMBuiltinFunction petal_vm_builtin_functions[];

/**
 * The number of built-in functions that are available in the petal_vm_builtin_functions array.
 */
extern const size_t petal_vm_builtin_functions_count;
