#include "codegen.h"
#include "ast/node.h"
#include "ast/node/function_declaration.h"
#include "codegen/context.h"
#include "codegen/result.h"
#include "core/diagnostic.h"
#include "core/type/type.h"
#include "core/type/value.h"
#include "util/defer.h"
#include "util/format.h"
#include "util/logger.h"
#include "util/vector.h"
#include "llvm-c/Core.h"
#include <llvm-c/Types.h>

// Forward declarations.
LLVMValueRef codegen_generate_statement(Codegen* codegen, Node* node);
LLVMValueRef codegen_generate_function_declaration(Codegen* codegen, FunctionDeclarationNode* node);

LLVMTypeRef codegen_type_to_llvm_type(Codegen* codegen, Type* type);

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
    for (size_t i = 0; i < codegen->nodes->length; i++) {
        if (!codegen_generate_statement(codegen, vector_get(codegen->nodes, i))) {
            return (CodegenResult){.status = CODEGEN_RESULT_FAILURE};
        }
    }

    // FIXME: Remove this when emitting modules is done.
    LLVMDumpModule(codegen->llvm_module);

    return (CodegenResult){.status = CODEGEN_RESULT_SUCCESS};
}

LLVMValueRef codegen_generate_statement(Codegen* codegen, Node* node) {
    switch (node->kind) {
    case NODE_KIND_FUNCTION_DECLARATION:
        return codegen_generate_function_declaration(codegen, (FunctionDeclarationNode*)node);

    default:
        auto node_string defer(free_str) = node_to_string(node);
        vector_append(
            codegen->diagnostics,
            diagnostic_create(node->position, format_string("unable to generate code for node: '%s'", node_string))
        );

        return nullptr;
    }
}

LLVMValueRef codegen_generate_function_declaration(Codegen* codegen, FunctionDeclarationNode* node) {
    auto return_type = codegen_type_to_llvm_type(codegen, node->return_type);
    if (!return_type) {
        return nullptr;
    }

    LLVMTypeRef parameters[node->parameters.length] = {};
    for (size_t i = 0; i < node->parameters.length; i++) {
        auto parameter = vector_get(&node->parameters, i);
        auto type = codegen_type_to_llvm_type(codegen, parameter.value_type);
        if (!type) {
            return nullptr;
        }

        parameters[i] = type;
    }

    auto function_type = LLVMFunctionType(return_type, parameters, node->parameters.length, false);
    auto function = LLVMAddFunction(codegen->llvm_module, node->name, function_type);

    // All functions must have an entry block.
    // We can then generate statements within that block.
    auto entry = LLVMAppendBasicBlockInContext(codegen->llvm_context, function, "entry");
    LLVMPositionBuilderAtEnd(codegen->llvm_builder, entry);

    // TODO: Generate `alloca` + `store` for function parameters.
    //       Parameters should basically be treated as normal variables.

    for (size_t i = 0; i < node->body.length; i++) {
        if (!codegen_generate_statement(codegen, vector_get(&node->body, i))) {
            return nullptr;
        }
    }

    // TODO: If any blocks within the function do not have a terminator, add one if it is trivial to do so.
    return function;
}

LLVMTypeRef codegen_type_to_llvm_type(Codegen* codegen, Type* type) {
    if (type->kind != TYPE_KIND_VALUE) {
        auto type_string defer(free_str) = type_to_string(type);
        vector_append(
            codegen->diagnostics,
            diagnostic_create(type->position, format_string("unable to use type '%s' in code generation", type_string))
        );

        return nullptr;
    }

    // This is a value type, there should be a corresponding LLVM type for it.
    auto value_type = (ValueType*)type;
    switch (value_type->value_kind) {
    case VALUE_TYPE_KIND_I32:
        return LLVMInt32TypeInContext(codegen->llvm_context);

    case VALUE_TYPE_KIND_F64:
        return LLVMDoubleTypeInContext(codegen->llvm_context);

    case VALUE_TYPE_KIND_VOID:
        return LLVMVoidTypeInContext(codegen->llvm_context);

    case VALUE_TYPE_KIND_INVALID:
        auto type_string defer(free_str) = type_to_string(type);
        vector_append(
            codegen->diagnostics,
            diagnostic_create(
                type->position,
                format_string("type '%s' is not a valid value type (possible typechecker error!)", type_string)
            )
        );

        return nullptr;
    }
}

void codegen_destroy(Codegen* codegen) {
    LLVMDisposeBuilder(codegen->llvm_builder);
    LLVMDisposeModule(codegen->llvm_module);
    LLVMContextDispose(codegen->llvm_context);
}
