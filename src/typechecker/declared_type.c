#include "typechecker/declared_type.h"
#include "util/vector.h"
#include <string.h>

DeclaredType declared_type_create(char* name, Type* type) {
    return (DeclaredType){.name = name, .type = type};
}

DeclaredType* declared_type_find_by_name(DeclaredTypeVector types, char* name) {
    for (size_t i = 0; i < types.length; i++) {
        auto type = vector_get_ref(&types, i);
        if (strcmp(type->name, name) == 0) {
            return type;
        }
    }

    return nullptr;
}
