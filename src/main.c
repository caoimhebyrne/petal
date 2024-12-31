#include "arguments.h"
#include "codegen/llvm_codegen.h"
#include "logger.h"
#include "module.h"
#include "string/format_string.h"
#include <libgen.h>
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
    char* standard_library_path = 0;
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

        (Argument){
            .name = "stdlib-path",
            .short_name = 's',
            .type = ARGUMENT_TYPE_STRING,
            .message = "Use the standard library at <path>",
            .value = &standard_library_path,
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

    Module module = module_create(extra_arguments.data[0], standard_library_path);
    if (!module_compile(&module)) {
        return -1;
    }

    LOG_DEBUG("main", "compiled module '%s' with %zu dependencies", module.file_name, module.dependencies.length);

    if (!output_file_name) {
        LOG_WARNING("main", "no output file specified, skipping emit step");
        return -1;
    }

    char* object_file_name = format_string("%s.o", output_file_name);
    char* error_message = llvm_codegen_emit(module, object_file_name);
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
    // LLVMDisposeModule(module.llvm_ref);

    return 0;
}
