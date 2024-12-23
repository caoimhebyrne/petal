#ifndef __LLVM_CODEGEN_H__
#define __LLVM_CODEGEN_H__

#include "../ast/node.h"
#include <llvm-c/Types.h>

typedef struct {
    // The context that is being used for code generation.
    LLVMContextRef context;

    // The module being generated.
    LLVMModuleRef module;

    // The builder for this module.
    LLVMBuilderRef builder;

    // The NodeStream to use as source for code generation.
    NodeStream node_stream;
} LLVMCodegen;

// Initializes a new LLVM code generator.
// Parameters:
// - node_stream: The node stream to use as a source for code generation.
//                This LLVMCodegen instance will then take "ownership" of this node_stream, and
//                it will be destroyed when llvm_codegen_destroy is called.
LLVMCodegen llvm_codegen_create(char* filename, NodeStream node_stream);

// Generates LLVM bytecode from this code generator's node stream.
void llvm_codegen_generate(LLVMCodegen codegen);

// Generates LLVM bytecode for a single node.
void llvm_codegen_generate_node(LLVMCodegen codegen, Node* node);

// Destroys the provided LLVM code generator.
void llvm_codegen_destroy(LLVMCodegen codegen);

#endif // __LLVM_CODEGEN_H__
