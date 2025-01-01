#include "core/module.h"
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
    Module main_module = module_create(strdup(argv[1]));
    module_compile(&main_module);
    module_destroy(main_module);

    printf("success: compilation finished\n");

    return 0;
}
