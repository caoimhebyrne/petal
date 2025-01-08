#include "codegen.h"
#include "ast/node.h"
#include "ast/node/function_declaration.h"
#include "ast/node/identifier_reference.h"
#include "ast/node/number_literal.h"
#include "ast/node/return.h"
#include "ast/node/variable_declaration.h"
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
LLVMValueRef codegen_generate_variable_declaration(Codegen* codegen, VariableDeclarationNode* node);
LLVMValueRef codegen_generate_return(Codegen* codegen, ReturnNode* node);

LLVMValueRef codegen_generate_expression(Codegen* codegen, Node* node);
LLVMValueRef codegen_generate_number_literal(Codegen* codegen, NumberLiteralNode* node);
LLVMValueRef codegen_generate_identifier_reference(Codegen* codegen, IdentifierReferenceNode* node);

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

    case NODE_KIND_VARIABLE_DECLARATION:
        return codegen_generate_variable_declaration(codegen, (VariableDeclarationNode*)node);

    case NODE_KIND_RETURN:
        return codegen_generate_return(codegen, (ReturnNode*)node);

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

    // Re-initialize the context for this function.
    if (!codegen_context_initialize(&codegen->context)) {
        vector_append(
            codegen->diagnostics,
            diagnostic_create(
                node->header.position,
                format_string("internal code generator error: failed to initialize codegen context!")
            )
        );

        return nullptr;
    }

    // TODO: Generate `alloca` + `store` for function parameters.
    //       Parameters should basically be treated as normal variables.

    for (size_t i = 0; i < node->body.length; i++) {
        if (!codegen_generate_statement(codegen, vector_get(&node->body, i))) {
            codegen_context_destroy(&codegen->context);
            return nullptr;
        }
    }

    // TODO: If any blocks within the function do not have a terminator, add one if it is trivial to do so.

    // Destroy the context as it is not valid for any other function.
    codegen_context_destroy(&codegen->context);
    return function;
}

LLVMValueRef codegen_generate_variable_declaration(Codegen* codegen, VariableDeclarationNode* node) {
    auto variable_type = codegen_type_to_llvm_type(codegen, node->type);
    if (!variable_type) {
        return nullptr;
    }

    // 1. Create an alloca for this variable.
    auto declaration = LLVMBuildAlloca(codegen->llvm_builder, variable_type, node->name);

    // 2. Store this as our reference for this variable.
    auto variable = (Variable){.name = node->name, .value = declaration};
    vector_append(&codegen->context.variables, variable);

    // 3. Store the initial value into the memory allocated for this variable.
    auto value = codegen_generate_expression(codegen, node->value);
    if (!value) {
        return nullptr;
    }

    LLVMBuildStore(codegen->llvm_builder, value, declaration);
    return declaration;
}

LLVMValueRef codegen_generate_return(Codegen* codegen, ReturnNode* node) {
    if (!node->return_value) {
        return LLVMBuildRetVoid(codegen->llvm_builder);
    }

    auto value = codegen_generate_expression(codegen, node->return_value);
    if (!value) {
        return nullptr;
    }

    return LLVMBuildRet(codegen->llvm_builder, value);
}

LLVMValueRef codegen_generate_expression(Codegen* codegen, Node* node) {
    switch (node->kind) {
    case NODE_KIND_NUMBER_LITERAL:
        return codegen_generate_number_literal(codegen, (NumberLiteralNode*)node);

    case NODE_KIND_IDENTIFIER_REFERENCE:
        return codegen_generate_identifier_reference(codegen, (IdentifierReferenceNode*)node);

    default:
        auto node_string defer(free_str) = node_to_string(node);
        vector_append(
            codegen->diagnostics,
            diagnostic_create(node->position, format_string("unable to generate code for node: '%s'", node_string))
        );

        return nullptr;
    }
}

LLVMValueRef codegen_generate_number_literal(Codegen* codegen, NumberLiteralNode* node) {
    if (!node->type) {
        auto node_string defer(free_str) = node_to_string((Node*)node);

        vector_append(
            codegen->diagnostics,
            diagnostic_create(
                node->header.position,
                format_string("internal code generator error: no type associated with node: '%s'", node_string)
            )
        );

        return nullptr;
    }

    auto type = codegen_type_to_llvm_type(codegen, node->type);
    if (!type) {
        return nullptr;
    }

    if (node->is_float) {
        return LLVMConstReal(type, node->number);
    } else {
        return LLVMConstInt(type, node->integer, false);
    }
}

LLVMValueRef codegen_generate_identifier_reference(Codegen* codegen, IdentifierReferenceNode* node) {
    auto variable = variable_find_by_name(codegen->context.variables, node->identifier);
    if (!variable) {
        vector_append(
            codegen->diagnostics,
            diagnostic_create(
                node->header.position,
                format_string(
                    "undefined variable: '%s', this should've been caught by the typechecker!",
                    node->identifier
                )
            )
        );

        return nullptr;
    }

    auto type = LLVMGetAllocatedType(variable->value);
    return LLVMBuildLoad2(codegen->llvm_builder, type, variable->value, node->identifier);
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
