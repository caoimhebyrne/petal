#include "arguments.h"
#include "ast/ast.h"
#include "ast/node.h"
#include "codegen/llvm_codegen.h"
#include "diagnostics.h"
#include "lexer/lexer.h"
#include "lexer/token.h"
#include "logger.h"
#include "string/format_string.h"
#include <stdio.h>
#include <stdlib.h>

#ifndef VERSION
#define VERSION "local"
#endif

#define VERSION_MESSAGE                                                                                                \
    "Petal v" VERSION "\n"                                                                                             \
    "This project is licensed under the MIT license.\n"                                                                \
    "GitHub: https://github.com/caoimhebyrne/petal/\n"

int main(int argc, char** argv) {
    char* output_file_name = 0;
    char* input_file_name = 0;
    bool display_help = false;
    bool display_version = false;

    Argument arguments[] = {
        (Argument){
            .name = 'o',
            .type = ARGUMENT_TYPE_STRING,
            .message = "Place the output into <file>",
            .value = &output_file_name,
        },

        (Argument){
            .name = 'h',
            .type = ARGUMENT_TYPE_FLAG,
            .message = "Display this message",
            .value = &display_help,
        },

        (Argument){
            .name = 'v',
            .type = ARGUMENT_TYPE_FLAG,
            .message = "Display the version of the compiler",
            .value = &display_version,
        },
    };

    size_t arguments_length = sizeof(arguments) / sizeof(Argument);
    parse_arguments(argc, argv, arguments, arguments_length, &input_file_name);

    if (display_version) {
        printf(VERSION_MESSAGE);
        return 0;
    }

    if (display_help) {
        print_help_message(argv[0], arguments, arguments_length);
        return 0;
    }

    if (!input_file_name) {
        LOG_ERROR("main", "no input file(s) provided!");
        print_help_message(argv[0], arguments, arguments_length);

        return -1;
    }

    Lexer lexer;
    if (!lexer_initialize(&lexer, input_file_name)) {
        return -1;
    }

    TokenStream token_stream = lexer_parse(&lexer);
    if (lexer.diagnostics.length != 0) {
        token_stream_destroy(&token_stream);
        diagnostic_stream_print(&lexer.diagnostics, input_file_name);
        lexer_destroy(&lexer);

        return -1;
    }

    LOG_INFO("main", "parsed %zu token(s)", token_stream.length);

    AST ast;
    if (!ast_initialize(&ast, token_stream)) {
        return -1;
    }

    NodeStream node_stream = ast_parse(&ast);
    if (ast.diagnostics.length != 0) {
        node_stream_destroy(&node_stream);
        diagnostic_stream_print(&ast.diagnostics, input_file_name);
        ast_destroy(&ast);

        return -1;
    }

    ast_destroy(&ast);

    LLVMCodegen codegen = llvm_codegen_create(input_file_name, node_stream);
    llvm_codegen_generate(&codegen);

    if (codegen.diagnostics.length != 0) {
        diagnostic_stream_print(&codegen.diagnostics, input_file_name);
        llvm_codegen_destroy(&codegen);

        return -1;
    }

    if (output_file_name) {
        char* error_message = llvm_codegen_emit(&codegen, format_string("%s.o", output_file_name));
        if (error_message) {
            LOG_ERROR("main", "%s", error_message);
            return -1;
        }

        int linker_status = system(format_string("clang -fuse-ld=lld %s.o -o %s", output_file_name, output_file_name));
        if (linker_status != 0) {
            LOG_ERROR("main", "linker failed! (%d)", linker_status);
            return -1;
        }

        LOG_SUCCESS("binary created: %s", output_file_name);
    } else {
        LOG_WARNING("main", "no output file specified, skipping emit step");
    }

    llvm_codegen_destroy(&codegen);
    return 0;
}
