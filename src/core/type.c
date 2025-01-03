#include "type.h"
#include <stdlib.h>

UnresolvedType* type_create_unresolved(char* name) {
    UnresolvedType* type = malloc(sizeof(UnresolvedType));
    if (!type) {
        return nullptr;
    }

    type->header.kind = TYPE_KIND_UNRESOLVED;
    type->name = name;

    return type;
}

void type_destroy(Type* type) {
    switch (type->kind) {
    case TYPE_KIND_UNRESOLVED:
        auto unresolved_type = (UnresolvedType*)type;
        free(unresolved_type->name);

        break;
    }

    free(type);
}
