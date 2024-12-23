#ifndef __LLVM_CODEGEN_H__
#define __LLVM_CODEGEN_H__

#include "../ast/node.h"
#include "../ast/type.h"
#include "../diagnostics.h"
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

    // The diagnostic stream that errors are produced onto.
    DiagnosticStream diagnostics;
} LLVMCodegen;

// Initializes a new LLVM code generator.
// Parameters:
// - node_stream: The node stream to use as a source for code generation.
//                This LLVMCodegen instance will then take "ownership" of this node_stream, and
//                it will be destroyed when llvm_codegen_destroy is called.
LLVMCodegen llvm_codegen_create(char* filename, NodeStream node_stream);

// Generates LLVM bytecode from this code generator's node stream.
// If this code generator's diagnostic stream has a length greater than 0, the code generation
// was not successful.
void llvm_codegen_generate(LLVMCodegen* codegen);

// Generates LLVM bytecode for a single node.
// Returns:
// - A boolean indicating whether the node could be generated or not.
//   If false, generation should stop.
bool llvm_codegen_generate_node(LLVMCodegen* codegen, Node* node);

// Destroys the provided LLVM code generator.
void llvm_codegen_destroy(LLVMCodegen* codegen);

// Converts a Type to an LLVMTypeRef.
LLVMTypeRef llvm_codegen_type_to_ref(LLVMCodegen* codegen, Type type);

#endif // __LLVM_CODEGEN_H__
