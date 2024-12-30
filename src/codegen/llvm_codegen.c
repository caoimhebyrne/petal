#include "llvm_codegen.h"
#include "../ast/node/binary_operation.h"
#include "../ast/node/boolean_literal.h"
#include "../ast/node/function_declaration.h"
#include "../ast/node/identifier_reference.h"
#include "../ast/node/number_literal.h"
#include "../ast/node/return.h"
#include "../ast/node/string_literal.h"
#include "../ast/node/variable_declaration.h"
#include "../ast/node/variable_reassignment.h"
#include "../string/format_string.h"
#include "stored_values.h"
#include <llvm-c/Analysis.h>
#include <llvm-c/Core.h>
#include <llvm-c/TargetMachine.h>
#include <llvm-c/Types.h>
#include <stdbool.h>
#include <stdio.h>
#include <string.h>

// Forward declarations.
LLVMValueRef llvm_codegen_generate_value(LLVMCodegen* codegen, Node* node);
LLVMValueRef llvm_codegen_generate_statement(LLVMCodegen* codegen, Node* node);

LLVMValueRef llvm_codegen_generate_function_declaration(LLVMCodegen* codegen, FunctionDeclarationNode* node);
LLVMValueRef llvm_codegen_generate_variable_declaration(LLVMCodegen* codegen, VariableDeclarationNode* node);
LLVMValueRef llvm_codegen_generate_return(LLVMCodegen* codegen, ReturnNode* node);
LLVMValueRef llvm_codegen_generate_variable_reassignment(LLVMCodegen* codegen, VariableReassignmentNode* node);

LLVMValueRef llvm_codegen_generate_function_call(LLVMCodegen* codegen, FunctionCallNode* node, bool as_value);
LLVMValueRef llvm_codegen_generate_identifier_reference(LLVMCodegen* codegen, IdentifierReferenceNode* node);
LLVMValueRef llvm_codegen_generate_binary_operation(LLVMCodegen* codegen, BinaryOperationNode* node);

LLVMTypeRef llvm_codegen_type_to_ref(LLVMCodegen* codegen, ResolvedType* type, Position position);

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
        Node* node = codegen->node_stream.data[i];
        if (node->node_type != NODE_TYPE_ALIAS_DECLARATION) {
            if (!llvm_codegen_generate_statement(codegen, node)) {
                return;
            }
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

// Generates an LLVM value for a value-node.
// Parameters:
// - expected_type: The (optional) expected type for this value, may be used for literal coersion.
LLVMValueRef llvm_codegen_generate_value(LLVMCodegen* codegen, Node* node) {
    switch (node->node_type) {
    case NODE_FUNCTION_CALL:
        return llvm_codegen_generate_function_call(codegen, (FunctionCallNode*)node, true);

    case NODE_NUMBER_LITERAL: {
        NumberLiteralNode* number_literal = (NumberLiteralNode*)node;

        // If the number literal's type was not resolved for some reason, throw an error.
        if (!number_literal->type->is_resolved) {
            diagnostic_stream_push(&codegen->diagnostics, number_literal->position, true, "unresolved type: '%s'",
                                   type_to_string((Type*)number_literal->type));
            return 0;
        }

        ResolvedType* number_literal_type = (ResolvedType*)number_literal->type;
        LLVMTypeRef value_type = llvm_codegen_type_to_ref(codegen, number_literal_type, number_literal->position);
        if (!value_type) {
            return 0;
        }

        if (number_literal_type->kind == TYPE_KIND_FLOAT_32 || number_literal_type->kind == TYPE_KIND_FLOAT_64) {
            return LLVMConstReal(value_type, number_literal->value);
        }

        return LLVMConstInt(value_type, number_literal->value, false);
    }

    case NODE_BOOLEAN_LITERAL: {
        BooleanLiteralNode* boolean_literal = (BooleanLiteralNode*)node;

        LLVMTypeRef int_1_type = LLVMInt1TypeInContext(codegen->context);
        return LLVMConstInt(int_1_type, boolean_literal->value, false);
    }

    case NODE_STRING_LITERAL: {
        StringLiteralNode* string_literal = (StringLiteralNode*)node;
        return LLVMBuildGlobalStringPtr(codegen->builder, string_literal->value, "a");
    }

    case NODE_IDENTIFIER_REFERENCE:
        return llvm_codegen_generate_identifier_reference(codegen, (IdentifierReferenceNode*)node);

    case NODE_BINARY_OPERATION:
        return llvm_codegen_generate_binary_operation(codegen, (BinaryOperationNode*)node);

    default: {
        diagnostic_stream_push(&codegen->diagnostics, node->position, true, "unable to generate value for node: '%s'",
                               node_to_string(node));
        return 0;
    }
    }
}

// Generates an LLVM value for a statement-node.
LLVMValueRef llvm_codegen_generate_statement(LLVMCodegen* codegen, Node* node) {
    switch (node->node_type) {
    case NODE_FUNCTION_DECLARATION:
        return llvm_codegen_generate_function_declaration(codegen, (FunctionDeclarationNode*)node);

    case NODE_RETURN:
        return llvm_codegen_generate_return(codegen, (ReturnNode*)node);

    case NODE_FUNCTION_CALL:
        return llvm_codegen_generate_function_call(codegen, (FunctionCallNode*)node, false);

    case NODE_VARIABLE_DECLARATION:
        return llvm_codegen_generate_variable_declaration(codegen, (VariableDeclarationNode*)node);

    case NODE_VARIABLE_REASSIGNMENT:
        return llvm_codegen_generate_variable_reassignment(codegen, (VariableReassignmentNode*)node);

    default: {
        diagnostic_stream_push(&codegen->diagnostics, node->position, true,
                               "unable to generate statement for node: '%s'", node_to_string(node));
        return 0;
    }
    }
}

LLVMValueRef llvm_codegen_generate_function_declaration(LLVMCodegen* codegen, FunctionDeclarationNode* node) {
    LOG_DEBUG("llvm-codegen", "generating function '%s'", node->name);

    // Before generating the function, we should clear all stored value references.
    stored_values_destroy(&codegen->stored_values);
    stored_values_initialize(&codegen->stored_values, 1);

    LLVMTypeRef return_type = llvm_codegen_type_to_ref(codegen, (ResolvedType*)node->return_type, node->position);
    if (!return_type) {
        return 0;
    }

    LLVMTypeRef parameters[node->parameters.length] = {};
    for (size_t i = 0; i < node->parameters.length; i++) {
        Parameter parameter = node->parameters.data[i];
        parameters[i] = llvm_codegen_type_to_ref(codegen, (ResolvedType*)parameter.type, node->position);
    }

    LLVMTypeRef function_type = LLVMFunctionType(return_type, parameters, node->parameters.length, false);
    LLVMValueRef function = LLVMAddFunction(codegen->module, node->name, function_type);

    // All code generated from now on will be inside this block.
    if (!node->is_external) {
        LLVMBasicBlockRef entry = LLVMAppendBasicBlockInContext(codegen->context, function, "entry");
        LLVMPositionBuilderAtEnd(codegen->builder, entry);
    }

    // Set the name of the function parameter.
    for (size_t i = 0; i < node->parameters.length; i++) {
        Parameter node_parameter = node->parameters.data[i];

        LLVMValueRef parameter = LLVMGetParam(function, i);
        LLVMSetValueName2(parameter, node_parameter.name, strlen(node_parameter.name));

        if (!node->is_external) {
            LLVMTypeRef parameter_type = parameters[i];
            LOG_DEBUG("llvm-codegen", "building alloca for parameter '%s' with type '%s'", node_parameter.name,
                      LLVMPrintTypeToString(parameter_type));

            // In order to access this properly, we need to build an alloca and store for this parameter.
            LLVMValueRef alloca = LLVMBuildAlloca(codegen->builder, parameter_type, node_parameter.name);

            // We can then store the parameter reference into this alloca.
            LLVMBuildStore(codegen->builder, parameter, alloca);

            // The alloca can now be referenced through the stored value.
            StoredValue stored_value = stored_value_create(node_parameter.name, alloca);
            stored_values_append(&codegen->stored_values, stored_value);
        }
    }

    if (node->function_body) {
        for (size_t i = 0; i < node->function_body->body.length; i++) {
            Node* body_node = node->function_body->body.data[i];
            if (body_node->node_type != NODE_TYPE_ALIAS_DECLARATION) {
                if (!llvm_codegen_generate_statement(codegen, body_node)) {
                    return 0;
                }
            }
        }

        // All functions need to have a terminator (either return or unreachable).
        // If the last instruction in the last block of the function is not a return, we can
        // generate it if and only if the function's return type is void.
        LLVMBasicBlockRef block = LLVMGetLastBasicBlock(function);
        LLVMValueRef last_instruction = LLVMGetLastInstruction(block);

        bool needs_terminator = true;
        if (last_instruction) {
            LLVMOpcode opcode = LLVMGetInstructionOpcode(last_instruction);
            needs_terminator = opcode != LLVMRet && opcode != LLVMUnreachable;
        }

        if (needs_terminator) {
            ResolvedType* return_type = (ResolvedType*)node->return_type;
            if (return_type->kind == TYPE_KIND_VOID) {
                LOG_DEBUG("llvm-codegen", "generating terminator for '%s'", node->name);
                LLVMBuildRetVoid(codegen->builder);
            } else {
                diagnostic_stream_push(&codegen->diagnostics, (Position){}, true,
                                       "function '%s' does not return a value", node->name);

                return 0;
            }
        }
    }

    // I'm unsure if I need to call something like LLVMClearInsertionPosition(builder) after I generate the nodes,
    // so let this be a comment to future me saying sorry if this not being here ends up breaking something.
    return function;
}

LLVMValueRef llvm_codegen_generate_function_call(LLVMCodegen* codegen, FunctionCallNode* node, bool as_value) {
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

        LLVMValueRef value = llvm_codegen_generate_value(codegen, argument);
        if (value == 0) {
            return 0;
        }

        arguments[i] = value;
    }

    // This took me a bit to figure out...
    // The type of (LLVMTypeOf) a global (in this case, a function) is a pointer to a global.
    // LLVMGlobalGetValueType gets the type that LLVMTypeOf is pointing to.
    LLVMTypeRef function_type = LLVMGlobalGetValueType(callee);
    return LLVMBuildCall2(codegen->builder, function_type, callee, arguments, node->arguments.length,
                          as_value ? node->name : "");
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

    LLVMTypeRef type = LLVMGetAllocatedType(stored_value->value);
    LLVMValueRef load = LLVMBuildLoad2(codegen->builder, type, stored_value->value, node->name);
    return load;
}

LLVMValueRef llvm_codegen_generate_return(LLVMCodegen* codegen, ReturnNode* node) {
    if (node->value == 0) {
        LOG_DEBUG("llvm-codegen", "generating return statement without value");
        return LLVMBuildRetVoid(codegen->builder);
    }

    LOG_DEBUG("llvm-codegen", "generating return statement with value '%s'", node_to_string(node->value));

    LLVMValueRef value = llvm_codegen_generate_value(codegen, node->value);
    if (value == 0) {
        return 0;
    }

    return LLVMBuildRet(codegen->builder, value);
}

LLVMValueRef llvm_codegen_generate_variable_reassignment(LLVMCodegen* codegen, VariableReassignmentNode* node) {
    // The `alloca` for this variable must exist.
    StoredValue* variable = stored_values_find_by_name(codegen->stored_values, node->variable_name);
    if (!variable) {
        diagnostic_stream_push(&codegen->diagnostics, node->position, true,
                               "undeclared variable: '%s', possibly a code generation bug!", node->variable_name);
        return 0;
    }

    LLVMValueRef new_value = llvm_codegen_generate_value(codegen, node->value);
    if (!new_value) {
        return 0;
    }

    // Build a `store` for the new value.
    return LLVMBuildStore(codegen->builder, new_value, variable->value);
}

LLVMValueRef llvm_codegen_generate_variable_declaration(LLVMCodegen* codegen, VariableDeclarationNode* node) {
    LLVMTypeRef variable_type = llvm_codegen_type_to_ref(codegen, (ResolvedType*)node->type, node->position);
    if (!variable_type) {
        return 0;
    }

    // 1. Create an "alloca" for this variable.
    LOG_DEBUG("llvm-codegen", "generating variable declaration '%s'", node->name);
    LLVMValueRef variable_declaration = LLVMBuildAlloca(codegen->builder, variable_type, node->name);

    // 2. Convert the initial value for this variable into an LLVMValueRef.
    LLVMValueRef initial_value = llvm_codegen_generate_value(codegen, node->value);
    if (!initial_value) {
        return 0;
    }

    // 3. Store the initial value into the alloca.
    LLVMBuildStore(codegen->builder, initial_value, variable_declaration);

    // 4. Store this in the variable lookup table.
    StoredValue stored_value = {.value = variable_declaration, .name = node->name};
    stored_values_append(&codegen->stored_values, stored_value);

    return variable_declaration;
}

LLVMValueRef llvm_codegen_generate_binary_operation(LLVMCodegen* codegen, BinaryOperationNode* node) {
    LLVMValueRef left = llvm_codegen_generate_value(codegen, node->left);
    LLVMValueRef right = llvm_codegen_generate_value(codegen, node->right);

    LOG_DEBUG("llvm-codegen", "generating binary operation between '%s' and '%s'", LLVMPrintValueToString(left),
              LLVMPrintValueToString(right));

    ResolvedType* expected_type = (ResolvedType*)node->type;

    switch (node->operator_) {
    case OPERATOR_PLUS:
        switch (expected_type->kind) {
        case TYPE_KIND_FLOAT_32:
        case TYPE_KIND_FLOAT_64:
            return LLVMBuildFAdd(codegen->builder, left, right, "fadd");

        case TYPE_KIND_INT_8:
        case TYPE_KIND_INT_32:
        case TYPE_KIND_INT_64:
            return LLVMBuildAdd(codegen->builder, left, right, "add");

        default: {
            diagnostic_stream_push(&codegen->diagnostics, node->position, true, "unable to produce code for node: '%s'",
                                   node_to_string((Node*)node));
            return 0;
        }
        }

    case OPERATOR_MINUS:
        switch (expected_type->kind) {
        case TYPE_KIND_FLOAT_32:
        case TYPE_KIND_FLOAT_64:
            return LLVMBuildFSub(codegen->builder, left, right, "fsubtract");

        case TYPE_KIND_INT_8:
        case TYPE_KIND_INT_32:
        case TYPE_KIND_INT_64:
            return LLVMBuildSub(codegen->builder, left, right, "subtract");

        default: {
            diagnostic_stream_push(&codegen->diagnostics, node->position, true, "unable to produce code for node: '%s'",
                                   node_to_string((Node*)node));
            return 0;
        }
        }

    case OPERATOR_DIVIDE:
        switch (expected_type->kind) {
        case TYPE_KIND_FLOAT_32:
        case TYPE_KIND_FLOAT_64:
            return LLVMBuildFDiv(codegen->builder, left, right, "fdivide");

        case TYPE_KIND_INT_8:
        case TYPE_KIND_INT_32:
        case TYPE_KIND_INT_64:
            return LLVMBuildSDiv(codegen->builder, left, right, "sdivide");

        default: {
            diagnostic_stream_push(&codegen->diagnostics, node->position, true, "unable to produce code for node: '%s'",
                                   node_to_string((Node*)node));
            return 0;
        }
        }

    case OPERATOR_MULTIPLY:
        switch (expected_type->kind) {
        case TYPE_KIND_FLOAT_32:
        case TYPE_KIND_FLOAT_64:
            return LLVMBuildFMul(codegen->builder, left, right, "fmultiply");

        case TYPE_KIND_INT_8:
        case TYPE_KIND_INT_32:
        case TYPE_KIND_INT_64:
            return LLVMBuildMul(codegen->builder, left, right, "multiply");

        default: {
            diagnostic_stream_push(&codegen->diagnostics, node->position, true, "unable to produce code for node: '%s'",
                                   node_to_string((Node*)node));
            return 0;
        }
        }

    default: {
        Diagnostic diagnostic = {
            .position = node->position,
            .is_terminal = true,
            .message =
                format_string("unsupported operator for binary operation: '%s'", operator_to_string(node->operator_)),
        };

        diagnostic_stream_append(&codegen->diagnostics, diagnostic);
        return 0;
    }
    }
}

void llvm_codegen_destroy(LLVMCodegen* codegen) {
    LLVMDisposeBuilder(codegen->builder);
    LLVMDisposeModule(codegen->module);
    LLVMContextDispose(codegen->context);

    node_stream_destroy(&codegen->node_stream);
}

LLVMTypeRef llvm_codegen_type_to_ref(LLVMCodegen* codegen, ResolvedType* type, Position position) {
    LLVMTypeRef type_ref;

    switch (type->kind) {
    case TYPE_KIND_BOOL:
        type_ref = LLVMInt1TypeInContext(codegen->context);
        break;

    case TYPE_KIND_INT_8:
        type_ref = LLVMInt8TypeInContext(codegen->context);
        break;

    case TYPE_KIND_INT_32:
        type_ref = LLVMInt32TypeInContext(codegen->context);
        break;

    case TYPE_KIND_INT_64:
        type_ref = LLVMInt64TypeInContext(codegen->context);
        break;

    case TYPE_KIND_FLOAT_32:
        type_ref = LLVMFloatTypeInContext(codegen->context);
        break;

    case TYPE_KIND_FLOAT_64:
        type_ref = LLVMDoubleTypeInContext(codegen->context);
        break;

    case TYPE_KIND_VOID:
        type_ref = LLVMVoidTypeInContext(codegen->context);
        break;

    default: {
        Diagnostic diagnostic = {
            .position = position,
            .is_terminal = true,
            .message = format_string("unable to convert type '%s' into llvm type", type_to_string((Type*)type)),
        };

        diagnostic_stream_append(&codegen->diagnostics, diagnostic);
        return 0;
    }
    }

    if (type->is_pointer) {
        return LLVMPointerType(type_ref, 0);
    } else {
        return type_ref;
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
