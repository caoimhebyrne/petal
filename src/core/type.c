#include "type.h"
#include "util/format.h"
#include <stdlib.h>

UnresolvedType* type_create_unresolved(Position position, char* name) {
    UnresolvedType* type = malloc(sizeof(UnresolvedType));
    if (!type) {
        return nullptr;
    }

    type->header.kind = TYPE_KIND_UNRESOLVED;
    type->header.position = position;
    type->name = name;

    return type;
}

char* type_to_string(Type* type) {
    switch (type->kind) {
    case TYPE_KIND_UNRESOLVED:
        return format_string("UnresolvedType ('%s')", ((UnresolvedType*)type)->name);
    }
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
