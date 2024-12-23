#include "llvm_codegen.h"
#include "../ast/node/function_declaration.h"
#include <llvm-c/Core.h>
#include <llvm-c/Types.h>
#include <string.h>

LLVMCodegen llvm_codegen_create(char* filename, NodeStream node_stream) {
    LLVMContextRef context = LLVMContextCreate();
    LLVMModuleRef module = LLVMModuleCreateWithNameInContext("module", context);
    LLVMBuilderRef builder = LLVMCreateBuilderInContext(context);

    LLVMSetSourceFileName(module, filename, strlen(filename));

    LOG_DEBUG("llvm-codegen", "created module for code generation");

    return (LLVMCodegen){context, module, builder, node_stream};
}

void llvm_codegen_generate(LLVMCodegen codegen) {
    for (size_t i = 0; i < codegen.node_stream.length; i++) {
        llvm_codegen_generate_node(codegen, codegen.node_stream.data[i]);
    }

    LOG_DEBUG("llvm-codegen", "dumping module:");
    LLVMDumpModule(codegen.module);
}

void llvm_codegen_generate_node(LLVMCodegen codegen, Node* node) {
    (void)codegen;
    switch (node->node_type) {
    case NODE_FUNCTION_DECLARATION: {
        FunctionDeclarationNode* function_declaration = (FunctionDeclarationNode*)node;

        LLVMTypeRef function_type = LLVMFunctionType(LLVMVoidTypeInContext(codegen.context), 0, 0, false);
        LLVMValueRef function = LLVMAddFunction(codegen.module, function_declaration->name, function_type);

        // FIXME: In order to ensure that code is actually generated, the builder needs to be pointed at a block within
        // this function.
        (void)function;

        break;
    }

    default:
        break;
    }
}

void llvm_codegen_destroy(LLVMCodegen codegen) {
    LLVMDisposeBuilder(codegen.builder);
    LLVMDisposeModule(codegen.module);
    LLVMContextDispose(codegen.context);

    node_stream_destroy(&codegen.node_stream);
}
