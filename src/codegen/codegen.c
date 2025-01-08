#include "codegen.h"
#include "codegen/context.h"
#include "codegen/result.h"
#include "core/diagnostic.h"
#include "core/position.h"
#include "util/format.h"
#include "util/logger.h"
#include "util/vector.h"
#include "llvm-c/Core.h"

Codegen codegen_create(NodeVector* nodes, DiagnosticVector* diagnostics) {
    auto context = (CodegenContext){};
    return (Codegen){
        nodes,
        diagnostics,
        context,
        .llvm_builder = 0,
        .llvm_context = 0,
        .llvm_module = 0,
    };
}

bool codegen_initialize(Codegen* codegen) {
    codegen->llvm_context = LLVMContextCreate();
    if (!codegen->llvm_context) {
        return false;
    }

    codegen->llvm_module = LLVMModuleCreateWithNameInContext("module", codegen->llvm_context);
    if (!codegen->llvm_module) {
        return false;
    }

    codegen->llvm_builder = LLVMCreateBuilderInContext(codegen->llvm_context);
    if (!codegen->llvm_builder) {
        return false;
    }

    LOG_DEBUG("codegen", "initialized llvm code generator context");
    return true;
}

CodegenResult codegen_generate(Codegen* codegen) {
    vector_append(
        codegen->diagnostics,
        diagnostic_create((Position){.length = 1}, format_string("code generation is not implemented yet"))
    );

    return (CodegenResult){.status = CODEGEN_RESULT_FAILURE};
}

void codegen_destroy(Codegen* codegen) {
    LLVMDisposeBuilder(codegen->llvm_builder);
    LLVMDisposeModule(codegen->llvm_module);
    LLVMContextDispose(codegen->llvm_context);
}
