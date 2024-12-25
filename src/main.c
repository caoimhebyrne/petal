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

typedef enum {
    // An option where a string value is expected.
    OPTION_TYPE_STRING,

    // An option where a boolean value is expected.
    // This value is produced by the flag existing or not.
    OPTION_TYPE_FLAG,
} OptionType;

typedef struct {
    // The name of this argument, must not be null.
    char name;

    // The type that this option's value should be, must not be null.
    OptionType type;

    // A simple help message associated with this option, must not be null.
    char* message;

    // A pointer to the variable to store this argument's value in, may be 0.
    void* value;
} Option;

void print_help_message(char* executable_name, Option* options, size_t options_length) {
    fprintf(stderr, "Usage: %s [options] file\n", executable_name);
    fprintf(stderr, "Options:\n");

    for (size_t i = 0; i < options_length; i++) {
        Option option = options[i];

        char* value_name = "";

        switch (option.type) {
        case OPTION_TYPE_STRING:
            value_name = " <string> ";
            break;
        case OPTION_TYPE_FLAG:
            break;
        }

        fprintf(stderr, "  -%c%-15s%s\n", option.name, value_name, option.message);
    }
}

void parse_options(size_t argc, char** argv, Option* options, size_t options_length, char** dangling_argument) {
    if (argc == 1) {
        // The first argument is the binary filename: there are no options to parse.
        return;
    }

    // Ignore the first argument, it is the program's name.
    for (size_t i = 1; i < argc; i++) {
        char* argument = argv[i];
        bool did_find_matching_option = false;

        for (size_t j = 0; j < options_length; j++) {
            Option option = options[j];

            if (argument[0] == '-') {
                if (argument[1] != option.name) {
                    // If the argument's name does not match the option's name, keep searching.
                    continue;
                }

                // If this is a flag, we do not need to check for a value.
                if (option.type == OPTION_TYPE_FLAG) {
                    bool* value_pointer = (bool*)option.value;
                    *value_pointer = true;

                    break;
                }

                // The next argument should be the value for this option.
                size_t value_index = i + 1;
                if (value_index > argc) {
                    // This argument has no value as we have ran out of values to parse.
                    break;
                }

                switch (option.type) {
                case OPTION_TYPE_FLAG: // Should not be reached.
                    break;

                case OPTION_TYPE_STRING: {
                    char** value_pointer = (char**)option.value;
                    *value_pointer = argv[value_index];

                    break;
                }
                }

                // Advancing the cursor to the value_index ensures that when this iteration is complete, the
                // value will not be parsed as an argument.
                i = value_index;

                // We can treat this argument as parsed.
                did_find_matching_option = true;
                break;
            }
        }

        if (did_find_matching_option) {
            continue;
        }

        if (*dangling_argument == 0) {
            *dangling_argument = argument;
        }
    }
}

int main(int argc, char** argv) {
    char* output_file_name = 0;
    char* input_file_name = 0;
    bool display_help = false;

    Option options[] = {
        (Option){
            .name = 'o',
            .type = OPTION_TYPE_STRING,
            .message = "Place the output into <file>",
            .value = &output_file_name,
        },

        (Option){
            .name = 'h',
            .type = OPTION_TYPE_FLAG,
            .message = "Displays this help message",
            .value = &display_help,
        },
    };

    size_t options_length = sizeof(options) / sizeof(Option);
    parse_options(argc, argv, options, options_length, &input_file_name);

    if (display_help) {
        print_help_message(argv[0], options, options_length);
        return 0;
    }

    if (!input_file_name) {
        fprintf(stderr, "error: no input file(s) provided!\n");
        print_help_message(argv[0], options, options_length);

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

        int linker_status = system(format_string("gcc -fuse-ld=lld %s.o -o %s", output_file_name, output_file_name));
        if (linker_status != 0) {
            LOG_INFO("main", "linker failed! (%d)", linker_status);
            return -1;
        }
    }

    llvm_codegen_destroy(&codegen);
    return 0;
}
