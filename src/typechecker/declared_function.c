#include "declared_function.h"

CREATE_STREAM(DeclaredFunctions, declared_functions, DeclaredFunction);

void declared_functions_destroy(DeclaredFunctions* functions) { free(functions->data); }

DeclaredFunction* declared_functions_find_by_name(DeclaredFunctions functions, char* name) {
    for (size_t i = 0; i < functions.length; i++) {
        DeclaredFunction* function = &functions.data[i];
        if (strcmp(function->name, name) == 0) {
            return function;
        }
    }

    return 0;
}
