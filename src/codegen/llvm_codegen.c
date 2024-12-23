#include "llvm_codegen.h"
#include "../ast/node/function_declaration.h"
#include "../ast/node/number_literal.h"
#include "../ast/node/return.h"
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
    switch (node->node_type) {
    case NODE_FUNCTION_DECLARATION: {
        FunctionDeclarationNode* function_declaration = (FunctionDeclarationNode*)node;
        LOG_DEBUG("llvm-codegen", "generating function '%s'", function_declaration->name);

        LLVMTypeRef return_type = llvm_codegen_type_to_ref(codegen, function_declaration->return_type);
        LLVMTypeRef function_type = LLVMFunctionType(return_type, 0, 0, false);
        LLVMValueRef function = LLVMAddFunction(codegen.module, function_declaration->name, function_type);

        LLVMBasicBlockRef entry = LLVMAppendBasicBlockInContext(codegen.context, function, "entry");
        LLVMPositionBuilderAtEnd(codegen.builder, entry);

        for (size_t i = 0; i < function_declaration->function_body.length; i++) {
            llvm_codegen_generate_node(codegen, function_declaration->function_body.data[i]);
        }

        break;
    }

    case NODE_RETURN: {
        ReturnNode* return_ = (ReturnNode*)node;
        if (return_->value == 0) {
            LOG_DEBUG("llvm-codegen", "generating return statement without value");
            LLVMBuildRetVoid(codegen.builder);
        } else {
            LOG_DEBUG("llvm-codegen", "generating return statement with value '%s'", node_to_string(return_->value));
            if (return_->value->node_type != NODE_NUMBER_LITERAL) {
                LOG_ERROR("llvm-codegen", "number literals are only supported in return expressions at the moment");
                break;
            }

            NumberLiteralNode* number_literal = (NumberLiteralNode*)return_->value;

            // FIXME: Probably want to infer this type somehow.
            //        If my function returns i64, it should be generating an i64 constant, etc.
            LLVMTypeRef int_32_type = LLVMInt32TypeInContext(codegen.context);
            LLVMBuildRet(codegen.builder, LLVMConstInt(int_32_type, (int32_t)number_literal->value, false));
        }

        break;
    }

    default:
        LOG_ERROR("llvm-codegen", "unsupported node: %s", node_to_string(node));
        break;
    }
}

void llvm_codegen_destroy(LLVMCodegen codegen) {
    LLVMDisposeBuilder(codegen.builder);
    LLVMDisposeModule(codegen.module);
    LLVMContextDispose(codegen.context);

    node_stream_destroy(&codegen.node_stream);
}

LLVMTypeRef llvm_codegen_type_to_ref(LLVMCodegen codegen, Type type) {
    switch (type) {
    case TYPE_INVALID:
        break;

    case TYPE_INT_32:
        return LLVMInt32TypeInContext(codegen.context);

    case TYPE_INT_64:
        return LLVMInt64TypeInContext(codegen.context);

    case TYPE_FLOAT_32:
        return LLVMFloatTypeInContext(codegen.context);

    case TYPE_VOID:
        return LLVMVoidTypeInContext(codegen.context);
    }

    LOG_ERROR("llvm-codegen", "unable to convert type '%s' into llvm type", type_to_string(type));

    // FIXME: The code generator should also have support for diagnotics.
    //        For now, emitting void is fine, since the compiler enforces that the switch statement above
    //        covers all cases (yet it still requires a return statement at the end of the function).
    return LLVMVoidTypeInContext(codegen.context);
}