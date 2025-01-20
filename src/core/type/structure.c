#include "core/type/structure.h"
#include "core/type/type.h"
#include "util/vector.h"
#include <stdlib.h>

StructureMember structure_member_create(char* name, Type* type) {
    return (StructureMember){.name = name, .type = type};
}

StructureType* structure_type_create(Position position) {
    StructureType* type = malloc(sizeof(StructureType));
    if (!type) {
        return nullptr;
    }

    type->header.kind = TYPE_KIND_STRUCTURE;
    type->header.position = position;
    type->members = (StructureMemberVector){};

    if (!vector_initialize(type->members, 1)) {
        free(type);
        return nullptr;
    }

    return type;
}
