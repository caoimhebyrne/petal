#include "core/module.h"
#include "options.h"
#include "util/arguments.h"
#include "util/defer.h"
#include "util/logger.h"
#include <stdio.h>
#include <string.h>

bool enable_debug_logging = false;

int main(int argc, char** argv) {
    ProgramOptions options;

    Argument arguments[] = {
        (Argument){
            .kind = ARGUMENT_KIND_FLAG,
            .name = "help",
            .short_name = 'h',
            .help_message = "Display this message",
            .value = &options.display_help,
        },

        (Argument){
            .kind = ARGUMENT_KIND_STRING,
            .name = "output",
            .short_name = 'o',
            .help_message = "Place the output into <file>",
            .value = &options.output_binary_name,
        },

        (Argument){
            .kind = ARGUMENT_KIND_FLAG,
            .name = "debug",
            .short_name = 'd',
            .help_message = "Enable debug logging",
            .value = &enable_debug_logging,
        }
    };

    char* input_file;
    parse_arguments(argc, argv, arguments, sizeof(arguments) / sizeof(Argument), &input_file);

    if (options.display_help) {
        print_help_message(argv[0], arguments, sizeof(arguments) / sizeof(Argument));
        return 0;
    }

    if (!input_file) {
        print_help_message(argv[0], arguments, sizeof(arguments) / sizeof(Argument));
        return -1;
    }

    // `module_create` expects the string passed to be `malloc`'d.
    // Arguments passed to the function through `argv` cannot be free'd, so to keep things similar between the
    // main module and the dependencies it resolves, we can duplicate the string, which allows it to be free'd when
    // the module is destroyed.
    defer(module_destroy) Module main_module = module_create(&options, strdup(input_file));
    if (!module_initialize(&main_module)) {
        fprintf(stderr, "Failed to initialize Petal compiler.");
        return -1;
    }

    if (!module_compile(&main_module)) {
        return -1;
    }

    LOG_SUCCESS("compilation finished");
    return 0;
}
