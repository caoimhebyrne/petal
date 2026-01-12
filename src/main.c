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

    if (!module_parse(&main_module)) {
        for (size_t i = 0; i < diagnostics.length; i++) {
            const Diagnostic diagnostic = diagnostics.data[i];
            log_error("diagnostic %zu: '%s' (module id = %zu)", i + 1, diagnostic.message, diagnostic.module_id);
        }

        return false;
    }

    allocator_clean(&allocator);

    return EXIT_SUCCESS;
}
