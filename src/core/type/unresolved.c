#include "core/type/unresolved.h"
#include <stdlib.h>

UnresolvedType* unresolved_type_create(Position position, char* name) {
    UnresolvedType* type = malloc(sizeof(UnresolvedType));
    if (!type) {
        return nullptr;
    }

    type->header.kind = TYPE_KIND_UNRESOLVED;
    type->header.position = position;
    type->name = name;

    return type;
}
