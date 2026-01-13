#include "allocator.h"
#include "diagnostic.h"
#include "logger.h"
#include "module.h"
#include <stdlib.h>

int main(const int argc, const char** argv, const char** envp) {
    (void)envp;

    if (argc != 2) {
        log_error("invalid usage, expected exactly one argument (path)");
        return EXIT_FAILURE;
    }

    const char* file_path = argv[1];
    if (!file_path) {
        log_error("invalid usage, expected exactly one argument (path)");
        return EXIT_FAILURE;
    }

    Allocator allocator = {0};
    allocator_init(&allocator);

    DiagnosticArray diagnostics = {0};
    diagnostic_array_init(&diagnostics, &allocator);

    Module main_module = {0};
    if (!module_init(&main_module, &allocator, &diagnostics, file_path)) {
        return false;
    }

    const bool module_parse_result = module_parse(&main_module);

    for (size_t i = 0; i < diagnostics.length; i++) {
        const Diagnostic diagnostic = diagnostics.data[i];

        fprintf(
            stderr,
            COLOR_RED "%s" COLOR_RESET COLOR_WHITE "(%.*s:%zu:%zu):" COLOR_RESET " %s\n",
            diagnostic_kind_string(diagnostic.kind),
            (int)main_module.file_path.length,
            main_module.file_path.data,
            diagnostic.position.line + 1,
            diagnostic.position.column + 1,
            diagnostic.message
        );
    }

    if (!module_parse_result) {
        return false;
    }

    allocator_clean(&allocator);

    return EXIT_SUCCESS;
}
