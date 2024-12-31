#include "arguments.h"
#include "ast/ast.h"
#include "ast/node.h"
#include "codegen/llvm_codegen.h"
#include "diagnostics.h"
#include "lexer/lexer.h"
#include "lexer/token.h"
#include "logger.h"
#include "typechecker/typechecker.h"
#include <llvm-c/Core.h>
#include <llvm-c/Linker.h>
#include <llvm-c/Types.h>
#include <stdio.h>
#include <stdlib.h>
#include <unistd.h>

// Attempts to compile a file into an LLVM module.
LLVMModuleRef compile_module(char* filename);

#define VERSION_MESSAGE                                                                                                \
    "Petal v" VERSION "\n"                                                                                             \
    "This project is licensed under the MIT license.\n"                                                                \
    "GitHub: https://github.com/caoimhebyrne/petal/\n"

int main(int argc, char** argv) {
    char* output_file_name = 0;
    bool display_help = false;
    bool display_version = false;

    ExtraArguments extra_arguments;
    extra_arguments_initialize(&extra_arguments, 1);

    Argument arguments[] = {
        (Argument){
            .name = "output",
            .short_name = 'o',
            .type = ARGUMENT_TYPE_STRING,
            .message = "Place the output into <file>",
            .value = &output_file_name,
        },

        (Argument){
            .name = "help",
            .short_name = 'h',
            .type = ARGUMENT_TYPE_FLAG,
            .message = "Display this message",
            .value = &display_help,
        },

        (Argument){
            .name = "version",
            .short_name = 'v',
            .type = ARGUMENT_TYPE_FLAG,
            .message = "Display the version of the compiler",
            .value = &display_version,
        },
    };

    size_t arguments_length = sizeof(arguments) / sizeof(Argument);
    parse_arguments(argc, argv, arguments, arguments_length, &extra_arguments);

    if (display_version) {
        printf(VERSION_MESSAGE);
        return 0;
    }

    if (display_help) {
        print_help_message(argv[0], arguments, arguments_length);
        return 0;
    }

    if (extra_arguments.length == 0) {
        LOG_ERROR("main", "no input file(s) provided!");
        print_help_message(argv[0], arguments, arguments_length);

        return -1;
    }

    LLVMModuleRef modules[extra_arguments.length];
    for (size_t i = 0; i < extra_arguments.length; i++) {
        LLVMModuleRef module = compile_module(extra_arguments.data[i]);
        if (!module) {
            return -1;
        }

        modules[i] = module;
    }

    if (!output_file_name) {
        LOG_WARNING("main", "no output file specified, skipping emit step");
        return -1;
    }

    LLVMModuleRef final_module;
    for (size_t i = 0; i < extra_arguments.length; i++) {
        LLVMModuleRef destination = modules[i++];
        LLVMModuleRef source = modules[i];

        LLVMLinkModules2(destination, source);
        final_module = destination;
    }

    char* object_file_name = format_string("%s.o", output_file_name);
    char* error_message = llvm_codegen_emit(final_module, object_file_name);
    if (error_message) {
        LOG_ERROR("main", "%s", error_message);
        return -1;
    }

    int status = system(format_string("clang -fuse-ld=lld -o %s %s", output_file_name, object_file_name));
    if (status != 0) {
        LOG_ERROR("main", "linker failed! (%d)", status);
        return -1;
    }

    LOG_SUCCESS("binary created: %s", output_file_name);
    LLVMDisposeModule(final_module);

    return 0;
}

LLVMModuleRef compile_module(char* filename) {
    Lexer lexer;
    if (!lexer_initialize(&lexer, filename)) {
        return 0;
    }

    TokenStream token_stream = lexer_parse(&lexer);
    if (lexer.diagnostics.length != 0) {
        token_stream_destroy(&token_stream);
        diagnostic_stream_print(&lexer.diagnostics, filename);
        lexer_destroy(&lexer);

        return 0;
    }

    AST ast;
    if (!ast_initialize(&ast, token_stream)) {
        return 0;
    }

    NodeStream node_stream = ast_parse(&ast);
    if (ast.diagnostics.length != 0) {
        node_stream_destroy(&node_stream);
        diagnostic_stream_print(&ast.diagnostics, filename);
        ast_destroy(&ast);

        return 0;
    }

    Typechecker typechecker = typechecker_create();
    typechecker_run(&typechecker, &node_stream);

    if (typechecker.diagnostics.length != 0) {
        diagnostic_stream_print(&typechecker.diagnostics, filename);
        typechecker_destroy(&typechecker);

        return 0;
    }

    typechecker_destroy(&typechecker);

    LLVMCodegen codegen = llvm_codegen_create(filename, node_stream);
    llvm_codegen_generate(&codegen);

    if (codegen.diagnostics.length != 0) {
        diagnostic_stream_print(&codegen.diagnostics, filename);

        llvm_codegen_destroy(&codegen);
        ast_destroy(&ast);
        lexer_destroy(&lexer);

        return 0;
    }

    // llvm_codegen_destroy(&codegen);
    ast_destroy(&ast);
    lexer_destroy(&lexer);

    return codegen.module;
}
