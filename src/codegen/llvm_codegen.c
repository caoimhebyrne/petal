#include "llvm_codegen.h"
#include "../ast/node/function_declaration.h"
#include "../ast/node/number_literal.h"
#include "../ast/node/return.h"
#include "../string/format_string.h"
#include <llvm-c/Core.h>
#include <llvm-c/Types.h>
#include <string.h>

LLVMCodegen llvm_codegen_create(char* filename, NodeStream node_stream) {
    LLVMContextRef context = LLVMContextCreate();
    LLVMModuleRef module = LLVMModuleCreateWithNameInContext("module", context);
    LLVMBuilderRef builder = LLVMCreateBuilderInContext(context);

    LLVMSetSourceFileName(module, filename, strlen(filename));

    DiagnosticStream diagnostics;
    diagnostic_stream_initialize(&diagnostics, 2);

    LOG_DEBUG("llvm-codegen", "created module for code generation");
    return (LLVMCodegen){context, module, builder, node_stream, diagnostics};
}

void llvm_codegen_generate(LLVMCodegen* codegen) {
    for (size_t i = 0; i < codegen->node_stream.length; i++) {
        if (!llvm_codegen_generate_node(codegen, codegen->node_stream.data[i])) {
            break;
        }
    }
}

bool llvm_codegen_generate_node(LLVMCodegen* codegen, Node* node) {
    switch (node->node_type) {
    case NODE_FUNCTION_DECLARATION: {
        if (!llvm_generate_function_declaration(codegen, (FunctionDeclarationNode*)node)) {
            return false;
        }

        break;
    }

    case NODE_RETURN: {
        if (!llvm_generate_return(codegen, (ReturnNode*)node)) {
            return false;
        }

        break;
    }

    default: {
        Diagnostic diagnostic = {
            .position = node->position,
            .is_terminal = true,
            .message = format_string("unable to generate code for node '%s'", node_to_string(node)),
        };

        diagnostic_stream_append(&codegen->diagnostics, diagnostic);
        return false;
    }
    }

    return true;
}

LLVMValueRef llvm_generate_function_declaration(LLVMCodegen* codegen, FunctionDeclarationNode* node) {
    LOG_DEBUG("llvm-codegen", "generating function '%s'", node->name);

    LLVMTypeRef return_type = llvm_codegen_type_to_ref(codegen, node->return_type);
    if (!return_type) {
        return 0;
    }

    // FIXME: Function declarations don't have support for parameters yet.
    LLVMTypeRef function_type = LLVMFunctionType(return_type, 0, 0, false);
    LLVMValueRef function = LLVMAddFunction(codegen->module, node->name, function_type);

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

LLVMValueRef llvm_generate_return(LLVMCodegen* codegen, ReturnNode* node) {
    if (node->value == 0) {
        LOG_DEBUG("llvm-codegen", "generating return statement without value");
        return LLVMBuildRetVoid(codegen->builder);
    }

    LOG_DEBUG("llvm-codegen", "generating return statement with value '%s'", node_to_string(node->value));

    // FIXME: Support for returning other nodes is not implemented yet.
    //        The easiest way to do this is probably to make all llvm_generate methods return a LLVMValueRef.
    if (node->value->node_type != NODE_NUMBER_LITERAL) {
        Diagnostic diagnostic = {
            .position = node->position,
            .is_terminal = true,
            .message = format_string("returning node '%s' is not supported yet", node_to_string(node->value)),
        };

        diagnostic_stream_append(&codegen->diagnostics, diagnostic);
        return 0;
    }

    NumberLiteralNode* number_literal = (NumberLiteralNode*)node->value;

    // FIXME: Probably want to infer this type somehow..?
    //        If my function returns i64, it should be generating an i64 constant, etc.
    LLVMTypeRef int_32_type = LLVMInt32TypeInContext(codegen->context);
    return LLVMBuildRet(codegen->builder, LLVMConstInt(int_32_type, (int32_t)number_literal->value, false));
}

void llvm_codegen_destroy(LLVMCodegen* codegen) {
    LLVMDisposeBuilder(codegen->builder);
    LLVMDisposeModule(codegen->module);
    LLVMContextDispose(codegen->context);

    node_stream_destroy(&codegen->node_stream);
}

LLVMTypeRef llvm_codegen_type_to_ref(LLVMCodegen* codegen, Type type) {
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
        // TODO: AST nodes do not have a position associated with them yet.
        .position = (Position){.line = 0, .column = 0, .index = 0},
        .is_terminal = true,
        .message = format_string("unable to convert type '%s' into llvm type", type_to_string(type)),
    };

    diagnostic_stream_append(&codegen->diagnostics, diagnostic);
    return 0;
}
