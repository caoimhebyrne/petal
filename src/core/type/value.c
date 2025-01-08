#include "core/type/value.h"
#include <stdlib.h>
#include <string.h>

ValueType* value_type_create(Position position, ValueTypeKind kind) {
    ValueType* type = malloc(sizeof(ValueType));
    if (!type) {
        return nullptr;
    }

    type->header.kind = TYPE_KIND_VALUE;
    type->header.position = position;
    type->value_kind = kind;

    return type;
}

ValueTypeKind value_type_kind_from_string(char* value) {
    if (strcmp(value, "void") == 0) {
        return VALUE_TYPE_KIND_VOID;
    } else if (strcmp(value, "i32") == 0) {
        return VALUE_TYPE_KIND_I32;
    } else if (strcmp(value, "f64") == 0) {
        return VALUE_TYPE_KIND_F64;
    }

    return VALUE_TYPE_KIND_INVALID;
}

const char* value_type_kind_to_string(ValueTypeKind kind) {
    switch (kind) {
    case VALUE_TYPE_KIND_INVALID:
        return "invalid";

    case VALUE_TYPE_KIND_VOID:
        return "void";

    case VALUE_TYPE_KIND_I32:
        return "i32";

    case VALUE_TYPE_KIND_F64:
        return "f64";
    }
}
