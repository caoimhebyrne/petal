#include "typechecker/declared_variable.h"
#include "util/vector.h"
#include <string.h>

DeclaredVariable declared_variable_create(char* name, Type* type) {
    return (DeclaredVariable){name, type};
}

DeclaredVariable* declared_variable_find_by_name(DeclaredVariableVector variables, char* name) {
    for (size_t i = 0; i < variables.length; i++) {
        auto variable = vector_get_ref(&variables, i);

        // FIXME: Use a hashtable here?
        if (strcmp(variable->name, name) == 0) {
            return variable;
        }
    }

    return nullptr;
}
