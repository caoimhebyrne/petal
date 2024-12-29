#include "declared_variable.h"

CREATE_STREAM(DeclaredVariables, declared_variables, DeclaredVariable);

void declared_variables_destroy(DeclaredVariables* stream) { free(stream->data); }

DeclaredVariable* declared_variables_find_by_name(DeclaredVariables variables, char* name) {
    for (size_t i = 0; i < variables.length; i++) {
        DeclaredVariable* variable = &variables.data[i];
        if (strcmp(variable->name, name) == 0) {
            return variable;
        }
    }

    return 0;
}
