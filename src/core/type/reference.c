#include "core/type/reference.h"
#include "core/type/type.h"
#include <stdlib.h>

ReferenceType* reference_type_create(Position position, Type* referenced_type) {
    ReferenceType* reference_type = malloc(sizeof(ReferenceType));
    if (!reference_type) {
        return nullptr;
    }

    reference_type->header.kind = TYPE_KIND_REFERENCE;
    reference_type->header.position = position;
    reference_type->referenced_type = referenced_type;

    return reference_type;
}
