#include "llvm_codegen.h"
#include "../ast/node/function_declaration.h"
#include "../ast/node/identifier_reference.h"
#include "../ast/node/number_literal.h"
#include "../ast/node/return.h"
#include "../string/format_string.h"
#include "stored_values.h"
#include <llvm-c/Analysis.h>
#include <llvm-c/Core.h>
#include <llvm-c/TargetMachine.h>
#include <llvm-c/Types.h>
#include <stdio.h>
#include <string.h>

LLVMCodegen llvm_codegen_create(char* filename, NodeStream node_stream) {
    LLVMContextRef context = LLVMContextCreate();
    LLVMModuleRef module = LLVMModuleCreateWithNameInContext("module", context);
    LLVMBuilderRef builder = LLVMCreateBuilderInContext(context);

    LLVMSetSourceFileName(module, filename, strlen(filename));

    DiagnosticStream diagnostics;
    diagnostic_stream_initialize(&diagnostics, 2);

    StoredValues stored_values;
    stored_values_initialize(&stored_values, 1);

    LOG_DEBUG("llvm-codegen", "created module for code generation");
    return (LLVMCodegen){context, module, builder, stored_values, node_stream, diagnostics};
}

void llvm_codegen_generate(LLVMCodegen* codegen) {
    for (size_t i = 0; i < codegen->node_stream.length; i++) {
        if (!llvm_codegen_generate_node(codegen, codegen->node_stream.data[i])) {
            return;
        }
    }

    LLVMDumpModule(codegen->module);

    char* error_message;
    bool failed = LLVMVerifyModule(codegen->module, LLVMReturnStatusAction, &error_message);
    if (failed) {
        Diagnostic diagnostic = {
            .position = (Position){.line = 0, .column = 0, .index = 0},
            .message = format_string("Module verification failed: %s", error_message),
            .is_terminal = true,
        };

        diagnostic_stream_append(&codegen->diagnostics, diagnostic);
        LLVMDisposeMessage(error_message);
    }
}

char* llvm_codegen_emit(LLVMCodegen* codegen, char* out_file_path) {
    char* error_message;
    char* host_triple = LLVMGetDefaultTargetTriple();

    LLVMInitializeAllTargetInfos();
    LLVMInitializeAllTargets();
    LLVMInitializeAllAsmPrinters();
    LLVMInitializeAllTargetMCs();

    LLVMTargetRef target;
    if (LLVMGetTargetFromTriple(host_triple, &target, &error_message)) {
        char* formatted_message = format_string("Failed to produce binary: %s", error_message);
        LLVMDisposeMessage(error_message);

        return formatted_message;
    }

    LLVMTargetMachineRef target_machine = LLVMCreateTargetMachine(target, host_triple, "", "", LLVMCodeGenLevelDefault,
                                                                  LLVMRelocPIC, LLVMCodeModelDefault);

    if (LLVMTargetMachineEmitToFile(target_machine, codegen->module, out_file_path, LLVMObjectFile, &error_message)) {
        char* formatted_message = format_string("Failed to produce binary: %s", error_message);
        LLVMDisposeMessage(error_message);

        return formatted_message;
    }

    return 0;
}

LLVMValueRef llvm_codegen_generate_node(LLVMCodegen* codegen, Node* node) {
    switch (node->node_type) {
    case NODE_FUNCTION_DECLARATION:
        return llvm_codegen_generate_function_declaration(codegen, (FunctionDeclarationNode*)node);

    case NODE_RETURN:
        return llvm_codegen_generate_return(codegen, (ReturnNode*)node);

    case NODE_FUNCTION_CALL:
        return llvm_codegen_generate_function_call(codegen, (FunctionCallNode*)node);

    case NODE_NUMBER_LITERAL: {
        NumberLiteralNode* number_literal = (NumberLiteralNode*)node;

        LLVMTypeRef int_32_type = LLVMInt32TypeInContext(codegen->context);
        return LLVMConstInt(int_32_type, (int32_t)number_literal->value, false);
    }

    case NODE_IDENTIFIER_REFERENCE:
        return llvm_codegen_generate_identifier_reference(codegen, (IdentifierReferenceNode*)node);

    default: {
        Diagnostic diagnostic = {
            .position = node->position,
            .is_terminal = true,
            .message = format_string("unable to generate code for node '%s'", node_to_string(node)),
        };

        diagnostic_stream_append(&codegen->diagnostics, diagnostic);
        return 0;
    }
    }
}

LLVMValueRef llvm_codegen_generate_function_declaration(LLVMCodegen* codegen, FunctionDeclarationNode* node) {
    LOG_DEBUG("llvm-codegen", "generating function '%s'", node->name);

    // Before generating the function, we should clear all stored value references.
    stored_values_destroy(&codegen->stored_values);
    stored_values_initialize(&codegen->stored_values, 1);

    LLVMTypeRef return_type = llvm_codegen_type_to_ref(codegen, node->return_type, node->position);
    if (!return_type) {
        return 0;
    }

    LLVMTypeRef parameters[node->parameters.length] = {};
    for (size_t i = 0; i < node->parameters.length; i++) {
        Parameter parameter = node->parameters.data[i];
        parameters[i] = llvm_codegen_type_to_ref(codegen, parameter.type, node->position);
    }

    LLVMTypeRef function_type = LLVMFunctionType(return_type, parameters, node->parameters.length, false);
    LLVMValueRef function = LLVMAddFunction(codegen->module, node->name, function_type);

    // Set the name of the function parameter.
    for (size_t i = 0; i < node->parameters.length; i++) {
        Parameter node_parameter = node->parameters.data[i];

        LLVMValueRef parameter = LLVMGetParam(function, i);
        LLVMSetValueName2(parameter, node_parameter.name, strlen(node_parameter.name));

        // Store this parameter within the stored values for this function.
        StoredValue stored_value = stored_value_create(node_parameter.name, parameter);
        stored_values_append(&codegen->stored_values, stored_value);
    }

    // If there are no nodes within this function's body, don't create a block.
    if (node->function_body.length == 0) {
        return function;
    }

    // All code generated from now on will be inside this block.
    LLVMBasicBlockRef entry = LLVMAppendBasicBlockInContext(codegen->context, function, "entry");
    LLVMPositionBuilderAtEnd(codegen->builder, entry);

    for (size_t i = 0; i < node->function_body.length; i++) {
        if (!llvm_codegen_generate_node(codegen, node->function_body.data[i])) {
            return 0;
        }
    }

    // I'm unsure if I need to call something like LLVMClearInsertionPosition(builder) after I generate the nodes,
    // so let this be a comment to future me saying sorry if this not being here ends up breaking something.
    return function;
}

LLVMValueRef llvm_codegen_generate_function_call(LLVMCodegen* codegen, FunctionCallNode* node) {
    LOG_DEBUG("llvm-codegen", "generating function call for '%s'", node->name);

    // In order to generate the call, we need the function itself.
    LLVMValueRef callee = LLVMGetNamedFunction(codegen->module, node->name);
    if (!callee) {
        Diagnostic diagnostic = {
            .position = node->position,
            .message = format_string("undefined function: '%s'", node->name),
            .is_terminal = true,
        };

        diagnostic_stream_append(&codegen->diagnostics, diagnostic);
        return 0;
    }

    LLVMValueRef arguments[node->arguments.length] = {};
    for (size_t i = 0; i < node->arguments.length; i++) {
        Node* argument = node->arguments.data[i];
        LLVMValueRef value = llvm_codegen_generate_node(codegen, argument);
        if (value == 0) {
            return 0;
        }

        arguments[i] = value;
    }

    // This took me a bit to figure out...
    // The type of (LLVMTypeOf) a global (in this case, a function) is a pointer to a global.
    // LLVMGlobalGetValueType gets the type that LLVMTypeOf is pointing to.
    LLVMTypeRef function_type = LLVMGlobalGetValueType(callee);
    return LLVMBuildCall2(codegen->builder, function_type, callee, arguments, node->arguments.length, node->name);
}

LLVMValueRef llvm_codegen_generate_identifier_reference(LLVMCodegen* codegen, IdentifierReferenceNode* node) {
    // Attempt to find a stored value with the provided name.
    StoredValue* stored_value = stored_values_find_by_name(codegen->stored_values, node->name);
    if (!stored_value) {
        Diagnostic diagnostic = {
            .position = node->position,
            .is_terminal = true,
            .message = format_string("undeclared variable: '%s'", node->name),
        };

        diagnostic_stream_append(&codegen->diagnostics, diagnostic);
        return 0;
    }

    return stored_value->value;
}

LLVMValueRef llvm_codegen_generate_return(LLVMCodegen* codegen, ReturnNode* node) {
    if (node->value == 0) {
        LOG_DEBUG("llvm-codegen", "generating return statement without value");
        return LLVMBuildRetVoid(codegen->builder);
    }

    LOG_DEBUG("llvm-codegen", "generating return statement with value '%s'", node_to_string(node->value));

    LLVMValueRef value = llvm_codegen_generate_node(codegen, node->value);
    if (value == 0) {
        return 0;
    }

    return LLVMBuildRet(codegen->builder, value);
}

void llvm_codegen_destroy(LLVMCodegen* codegen) {
    LLVMDisposeBuilder(codegen->builder);
    LLVMDisposeModule(codegen->module);
    LLVMContextDispose(codegen->context);

    node_stream_destroy(&codegen->node_stream);
}

LLVMTypeRef llvm_codegen_type_to_ref(LLVMCodegen* codegen, Type type, Position position) {
    switch (type) {
    case TYPE_INVALID:
        break;

    case TYPE_INT_32:
        return LLVMInt32TypeInContext(codegen->context);

    case TYPE_INT_64:
        return LLVMInt64TypeInContext(codegen->context);

    case TYPE_FLOAT_32:
        return LLVMFloatTypeInContext(codegen->context);

    case TYPE_VOID:
        return LLVMVoidTypeInContext(codegen->context);
    }

    Diagnostic diagnostic = {
        .position = position,
        .is_terminal = true,
        .message = format_string("unable to convert type '%s' into llvm type", type_to_string(type)),
    };

    diagnostic_stream_append(&codegen->diagnostics, diagnostic);
    return 0;
}
