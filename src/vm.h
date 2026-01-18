#pragma once

#include "allocator.h"
#include "array.h"
#include "ast_statement.h"
#include "vm_api.h"
#include "vm_value.h"
#include <stddef.h>

DEFINE_ARRAY_TYPE(FunctionDeclarationArray, function_declaration_array, FunctionDeclarationStatement)

/**
 * The state of the Petal virtual machine.
 */
typedef struct {
    /**
     * The exit code of the process once interpreting has been completed.
     */
    size_t exit_code;
} VMState;

/**
 * A variable declared within the scope of the VM.
 */
typedef struct {
    /**
     * The name of the variable.
     */
    StringBuffer name;

    /**
     * The value assigned to the variable.
     */
    VMValue value;
} VMVariable;

DEFINE_ARRAY_TYPE(VMVariableArray, vm_variable_array, VMVariable)

/**
 * A scope of execution within the VM.
 * A scope has input parameters, and may optionally have a value that is returned from it.
 */
typedef struct {
    /**
     * The value being returned out of this scope.
     */
    VMValue return_value;

    /**
     * Whether the scope should stop executing after the current statement.
     */
    bool stop_execution;

    /**
     * The variables defined within this scope.
     */
    VMVariableArray variables;
} VMScope;

/**
 * Initializes a new VM scope.
 */
void vm_scope_init(VMScope* scope, Allocator* allocator);

/**
 * A very barebones virtual machine for the Petal language.
 */
typedef struct {
    // The allocator.
    Allocator* allocator;

    // The state of the virtual machine.
    VMState state;

    // The function declarations available to be executed.
    FunctionDeclarationArray functions;
} PetalVM;

/**
 * The type of a built-in function handler.
 */
typedef VMValue (*VMBuiltinFunctionHandler)(PetalBuiltinContext* context);

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
 * Initializes a [PetalVM] with the provided top-level statements.
 */
void petal_vm_init(PetalVM* vm, Allocator* allocator, const StatementArray* statements);

/**
 * Attempts to find and execute the main method in the VM's current context.
 */
bool petal_vm_exec(PetalVM* vm);
