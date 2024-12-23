#include "llvm_codegen.h"
#include <llvm-c/Core.h>

LLVMCodegen llvm_codegen_create(NodeStream node_stream) {
    LLVMContextRef context = LLVMContextCreate();
    LLVMModuleRef module = LLVMModuleCreateWithNameInContext("module", context);
    LLVMBuilderRef builder = LLVMCreateBuilderInContext(context);

    LOG_DEBUG("llvm-codegen", "created module for code generation");

    return (LLVMCodegen){context, module, builder, node_stream, .position = 0};
}

void llvm_codegen_generate() { LOG_TODO("llvm-codegen", "generation is not implemented yet"); }

void llvm_codegen_destroy(LLVMCodegen codegen) {
    LLVMDisposeBuilder(codegen.builder);
    LLVMDisposeModule(codegen.module);
    LLVMContextDispose(codegen.context);

    node_stream_destroy(&codegen.node_stream);
}
