#pragma once

#include "ast/node.h"
#include "codegen/context.h"
#include "codegen/result.h"
#include "core/diagnostic.h"

// The code generator is responsible for communicating with its backend.
typedef struct {
    // A reference to the vector of nodes to type check.
    NodeVector* nodes;

    // A reference to a vector of diagnostics.
    DiagnosticVector* diagnostics;

    // The context for this code generator.
    CodegenContext context;

    // The current LLVM context.
    LLVMContextRef llvm_context;

    // The current LLVM module being compiled.
    LLVMModuleRef llvm_module;

    // The builder used for LLVM instructions.
    LLVMBuilderRef llvm_builder;
} Codegen;

// Creates a new Codegen.
// Paramaters:
// - nodes: A reference to the vector of nodes to type check.
// - diagnostics: A reference to a vector of diagnostics.
Codegen codegen_create(NodeVector* nodes, DiagnosticVector* diagnostics);

// Initializes a code generator.
bool codegen_initialize(Codegen* codegen);

// Generates code with this code generator's nodes.
// Parameters:
// - codegen: The code-generator instance to use.
// Returns: A codegen result.
CodegenResult codegen_generate(Codegen* codegen);

// Destroys a Codegen.
void codegen_destroy(Codegen* codegen);
