#include "codegen.h"
#include "ast/node.h"
#include "ast/node/binary_operation.h"
#include "ast/node/function_call.h"
#include "ast/node/function_declaration.h"
#include "ast/node/identifier_reference.h"
#include "ast/node/number_literal.h"
#include "ast/node/return.h"
#include "ast/node/variable_declaration.h"
#include "ast/node/variable_reassignment.h"
#include "codegen/context.h"
#include "codegen/result.h"
#include "core/diagnostic.h"
#include "core/position.h"
#include "core/type/reference.h"
#include "core/type/type.h"
#include "core/type/value.h"
#include "util/defer.h"
#include "util/format.h"
#include "util/logger.h"
#include "util/vector.h"
#include <llvm-c/Analysis.h>
#include <llvm-c/Core.h>
#include <llvm-c/Target.h>
#include <llvm-c/TargetMachine.h>
#include <llvm-c/Types.h>
#include <stdio.h>

// Forward declarations.
LLVMValueRef codegen_generate_statement(Codegen* codegen, Node* node);
LLVMValueRef codegen_generate_function_declaration(Codegen* codegen, FunctionDeclarationNode* node);
LLVMValueRef codegen_generate_variable_declaration(Codegen* codegen, VariableDeclarationNode* node);
LLVMValueRef codegen_generate_return(Codegen* codegen, ReturnNode* node);
LLVMValueRef codegen_generate_variable_reassignment(Codegen* codegen, VariableReassignmentNode* node);

LLVMValueRef codegen_generate_expression(Codegen* codegen, Node* node);
LLVMValueRef codegen_generate_number_literal(Codegen* codegen, NumberLiteralNode* node);
LLVMValueRef codegen_generate_identifier_reference(Codegen* codegen, IdentifierReferenceNode* node);
LLVMValueRef codegen_generate_binary_operation(Codegen* codegen, BinaryOperationNode* node);
LLVMValueRef codegen_generate_function_call(Codegen* codegen, FunctionCallNode* node, bool statement);

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
        auto node = vector_get(codegen->nodes, i);
        if (node->kind == NODE_KIND_TYPE_DECLARATION) {
            continue;
        }

        if (!codegen_generate_statement(codegen, node)) {
            return (CodegenResult){.status = CODEGEN_RESULT_FAILURE};
        }
    }

    if (enable_debug_logging) {
        LLVMDumpModule(codegen->llvm_module);
    }

    char* error_message;
    bool failed = LLVMVerifyModule(codegen->llvm_module, LLVMReturnStatusAction, &error_message);
    if (failed) {
        vector_append(
            codegen->diagnostics,
            diagnostic_create((Position){}, format_string("module verification failed: %s", error_message))
        );

        return (CodegenResult){.status = CODEGEN_RESULT_FAILURE};
    }

    LLVMDisposeMessage(error_message);
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

    case NODE_KIND_FUNCTION_CALL:
        return codegen_generate_function_call(codegen, (FunctionCallNode*)node, true);

    case NODE_KIND_VARIABLE_REASSIGNMENT:
        return codegen_generate_variable_reassignment(codegen, (VariableReassignmentNode*)node);

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

    LLVMTypeRef parameter_types[node->parameters.length] = {};
    for (size_t i = 0; i < node->parameters.length; i++) {
        auto parameter = vector_get(&node->parameters, i);
        auto type = codegen_type_to_llvm_type(codegen, parameter.value_type);
        if (!type) {
            return nullptr;
        }

        parameter_types[i] = type;
    }

    auto function_type = LLVMFunctionType(return_type, parameter_types, node->parameters.length, false);
    auto function = LLVMAddFunction(codegen->llvm_module, node->name, function_type);

    if ((node->modifiers & FUNCTION_MODIFIER_EXTERN) == FUNCTION_MODIFIER_EXTERN) {
        LLVMSetLinkage(function, LLVMExternalLinkage);
    } else {
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

        for (size_t i = 0; i < node->parameters.length; i++) {
            auto parameter = vector_get(&node->parameters, i);
            auto llvm_parameter = LLVMGetParam(function, i);

            // The parameter's type must be resolvable.
            auto parameter_type = codegen_type_to_llvm_type(codegen, parameter.value_type);
            if (!parameter_type) {
                return nullptr;
            }

            // Allocate a stack variable for this parameter.
            auto declaration = LLVMBuildAlloca(codegen->llvm_builder, parameter_type, parameter.name);

            // Store the parameter's value into this local variable.
            LLVMBuildStore(codegen->llvm_builder, llvm_parameter, declaration);

            if (parameter.value_type->kind == TYPE_KIND_REFERENCE) {
                // The previous store was for the pointer __value__, now we are loading what that pointer points *to*.
                auto load = LLVMBuildLoad2(codegen->llvm_builder, parameter_type, declaration, parameter.name);

                // In order to make the rest of the codegen aware of this parameter, we must treat it as a stored
                // variable in the function's context.
                auto variable = (Variable){.name = parameter.name, .value = load};
                vector_append(&codegen->context.variables, variable);
            } else {
                // In order to make the rest of the codegen aware of this parameter, we must treat it as a stored
                // variable in the function's context.
                auto variable = (Variable){.name = parameter.name, .value = declaration};
                vector_append(&codegen->context.variables, variable);
            }
        }

        for (size_t i = 0; i < node->body.length; i++) {
            auto body_node = vector_get(&node->body, i);
            if (body_node->kind == NODE_KIND_TYPE_DECLARATION) {
                continue;
            }

            if (!codegen_generate_statement(codegen, body_node)) {
                codegen_context_destroy(&codegen->context);
                return nullptr;
            }
        }

        // Destroy the context as it is not valid for any other function.
        codegen_context_destroy(&codegen->context);
    }

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

LLVMValueRef codegen_generate_variable_reassignment(Codegen* codegen, VariableReassignmentNode* node) {
    auto value = codegen_generate_expression(codegen, node->value);
    if (!value) {
        return nullptr;
    }

    auto variable = variable_find_by_name(codegen->context.variables, node->name);
    if (!variable) {
        vector_append(
            codegen->diagnostics,
            diagnostic_create(
                node->header.position,
                format_string("possible typechecker bug: '%s' is not declared during re-assignment", node->name)
            )
        );

        return nullptr;
    }

    return LLVMBuildStore(codegen->llvm_builder, value, variable->value);
}

LLVMValueRef codegen_generate_expression(Codegen* codegen, Node* node) {
    switch (node->kind) {
    case NODE_KIND_NUMBER_LITERAL:
        return codegen_generate_number_literal(codegen, (NumberLiteralNode*)node);

    case NODE_KIND_IDENTIFIER_REFERENCE:
        return codegen_generate_identifier_reference(codegen, (IdentifierReferenceNode*)node);

    case NODE_KIND_BINARY_OPERATION:
        return codegen_generate_binary_operation(codegen, (BinaryOperationNode*)node);

    case NODE_KIND_FUNCTION_CALL:
        return codegen_generate_function_call(codegen, (FunctionCallNode*)node, false);

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

    if (node->value_type && node->value_type->kind == TYPE_KIND_REFERENCE) {
        return variable->value;
    }

    auto type = LLVMGetAllocatedType(variable->value);
    return LLVMBuildLoad2(codegen->llvm_builder, type, variable->value, node->identifier);
}

LLVMValueRef codegen_generate_binary_operation(Codegen* codegen, BinaryOperationNode* node) {
    // The left side must have a value.
    auto left_value = codegen_generate_expression(codegen, node->left);
    if (!left_value) {
        return nullptr;
    }

    // The right side must have a value.
    auto right_value = codegen_generate_expression(codegen, node->right);
    if (!right_value) {
        return nullptr;
    }

    switch (node->operator) {
    case OPERATOR_ADD:
        return LLVMBuildAdd(codegen->llvm_builder, left_value, right_value, "add");

    case OPERATOR_SUBTRACT:
        return LLVMBuildSub(codegen->llvm_builder, left_value, right_value, "subtract");

    case OPERATOR_MULTIPLY:
        return LLVMBuildMul(codegen->llvm_builder, left_value, right_value, "multiply");

    case OPERATOR_DIVIDE:
        return LLVMBuildSDiv(codegen->llvm_builder, left_value, right_value, "divide");
    }
}

LLVMValueRef codegen_generate_function_call(Codegen* codegen, FunctionCallNode* node, bool statement) {
    auto function = LLVMGetNamedFunction(codegen->llvm_module, node->function_name);
    if (!function) {
        vector_append(
            codegen->diagnostics,
            diagnostic_create(
                node->header.position,
                format_string(
                    "undefined function: '%s', this should've been caught by the typechecker!",
                    node->function_name
                )
            )
        );

        return nullptr;
    }

    auto function_type = LLVMGlobalGetValueType(function);

    LLVMValueRef arguments[node->arguments.length] = {};
    for (size_t i = 0; i < node->arguments.length; i++) {
        auto value = codegen_generate_expression(codegen, vector_get(&node->arguments, i));
        if (!value) {
            return nullptr;
        }

        arguments[i] = value;
    }

    return LLVMBuildCall2(
        codegen->llvm_builder,
        function_type,
        function,
        arguments,
        node->arguments.length,
        statement ? "" : node->function_name
    );
}

LLVMTypeRef codegen_type_to_llvm_type(Codegen* codegen, Type* type) {
    if (type->kind == TYPE_KIND_REFERENCE) {
        auto reference_type = (ReferenceType*)type;
        if (!reference_type->referenced_type) {
            vector_append(
                codegen->diagnostics,
                diagnostic_create(type->position, format_string("reference type had no referenced type?"))
            );

            return nullptr;
        }

        auto llvm_type = codegen_type_to_llvm_type(codegen, reference_type->referenced_type);
        if (!llvm_type) {
            return nullptr;
        }

        // Unsure what AddressSpace is, but zero seems to work, some docs reference it as the "default" anyway.
        return LLVMPointerType(llvm_type, 0);
    }

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

bool codegen_emit_object(Codegen* codegen, char* file_name) {
    LOG_DEBUG("llvm-codegen", "emitting object to file: '%s'", file_name);

    auto host_triple = LLVMGetDefaultTargetTriple();

    LLVMInitializeAllTargetInfos();
    LLVMInitializeAllTargets();
    LLVMInitializeAllAsmPrinters();
    LLVMInitializeAllTargetMCs();

    char* error_message;
    LLVMTargetRef target;
    if (LLVMGetTargetFromTriple(host_triple, &target, &error_message)) {
        auto formatted_message = format_string("%s", error_message);
        LLVMDisposeMessage(error_message);

        fprintf(stderr, "error: %s", formatted_message);
        return false;
    }

    auto target_machine = LLVMCreateTargetMachine(
        target,
        host_triple,
        "",
        "",
        LLVMCodeGenLevelDefault,
        LLVMRelocPIC,
        LLVMCodeModelDefault
    );

    auto emit_failed = LLVMTargetMachineEmitToFile(
        target_machine,
        codegen->llvm_module,
        file_name,
        LLVMObjectFile,
        &error_message
    );

    if (emit_failed) {
        auto formatted_message = format_string("%s", error_message);
        LLVMDisposeMessage(error_message);

        fprintf(stderr, "error: %s", formatted_message);
        return false;
    }

    LLVMDisposeTargetMachine(target_machine);
    LLVMDisposeMessage(host_triple);
    return true;
}

void codegen_destroy(Codegen* codegen) {
    LLVMDisposeBuilder(codegen->llvm_builder);
    LLVMDisposeModule(codegen->llvm_module);
    LLVMContextDispose(codegen->llvm_context);
}
