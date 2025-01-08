#pragma once

#include "util/vector.h"
#include <llvm-c/Core.h>
#include <llvm-c/Types.h>

typedef struct {
    // The name of this variable.
    char* name;

    // The LLVM value for this variable.
    LLVMValueRef value;
} Variable;

typedef Vector(Variable) VariableVector;

typedef struct {
    VariableVector variables;
} CodegenContext;

// Initializes a CodegenContext.
// Parameters:
// - context: A reference to the codegen context to initialize.
// Returns whether initialization was successful.
bool codegen_context_initialize(CodegenContext* context);

// Destroys a CodegenContext.
// Parameters:
// - context: A reference to the codegen context to destroy.
// Returns whether destruction was successful.
void codegen_context_destroy(CodegenContext* context);
