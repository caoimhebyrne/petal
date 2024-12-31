#ifndef __LLVM_CODEGEN_H__
#define __LLVM_CODEGEN_H__

#include "../ast/node.h"
#include "../ast/node/binary_operation.h"
#include "../ast/node/function_call.h"
#include "../ast/node/function_declaration.h"
#include "../ast/node/identifier_reference.h"
#include "../ast/node/return.h"
#include "../ast/node/variable_declaration.h"
#include "../ast/type.h"
#include "../diagnostics.h"
#include "../module.h"
#include "stored_values.h"
#include <llvm-c/Types.h>

typedef struct {
    // The context that is being used for code generation.
    LLVMContextRef context;

    // The module being generated.
    LLVMModuleRef module;

    // The builder for this module.
    LLVMBuilderRef builder;

    // The stored values within this code generator.
    StoredValues stored_values;

    // The NodeStream to use as source for code generation.
    NodeStream node_stream;

    // The diagnostic stream that errors are produced onto.
    DiagnosticStream diagnostics;
} LLVMCodegen;

// Initializes a new LLVM code generator.
// Parameters:
// - node_stream: The node stream to use as a source for code generation.
//                This LLVMCodegen instance will then take "ownership" of this node_stream, and
//                it will be destroyed when llvm_codegen_destroy is called.
LLVMCodegen llvm_codegen_create(LLVMContextRef context, char* filename, NodeStream node_stream);

// Generates LLVM bytecode from this code generator's node stream.
// If this code generator's diagnostic stream has a length greater than 0, the code generation
// was not successful.
void llvm_codegen_generate(LLVMCodegen* codegen);

// Emits an object file from the generated LLVM bytecode.
// Parameters:
// - codegen: The code generator being used.
// - out_file_path: The path (relative to the current working directory) that the object file should be written to.
// Returns:
// - An error message if one occurred, otherwise 0.
char* llvm_codegen_emit(Module module, char* out_file_path);

// Destroys the provided LLVM code generator.
void llvm_codegen_destroy(LLVMCodegen* codegen);

#endif // __LLVM_CODEGEN_H__
