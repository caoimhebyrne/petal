#include "typechecker/declared_function.h"
#include "util/vector.h"
#include <string.h>

DeclaredFunction declared_function_create(char* name, Type* return_type, ParameterVector* parameters) {
    return (DeclaredFunction){name, return_type, parameters};
}

DeclaredFunction* declared_function_find_by_name(DeclaredFunctionVector functions, char* name) {
    for (size_t i = 0; i < functions.length; i++) {
        auto variable = vector_get_ref(&functions, i);

        // FIXME: Use a hashtable here?
        if (strcmp(variable->name, name) == 0) {
            return variable;
        }
    }

    return nullptr;
}
