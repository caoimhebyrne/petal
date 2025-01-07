#include "core/module.h"
#include "util/defer.h"
#include "util/logger.h"
#include <stdio.h>
#include <string.h>

int main(int argc, char** argv) {
    if (argc != 2) {
        fprintf(stderr, "Usage: %s [file]\n", argv[0]);
        return -1;
    }

    // `module_create` expects the string passed to be `malloc`'d.
    // Arguments passed to the function through `argv` cannot be free'd, so to keep things similar between the
    // main module and the dependencies it resolves, we can duplicate the string, which allows it to be free'd when
    // the module is destroyed.
    defer(module_destroy) Module main_module = module_create(strdup(argv[1]));
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
