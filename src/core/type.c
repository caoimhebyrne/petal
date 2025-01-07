#include "type.h"
#include "util/format.h"
#include <stdlib.h>
#include <string.h>

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

ValueType* type_create_value(Position position, ValueTypeKind kind) {
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
    if (strcmp(value, "i32") == 0) {
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

    case VALUE_TYPE_KIND_I32:
        return "i32";

    case VALUE_TYPE_KIND_F64:
        return "f64";
    }
}

char* type_to_string(Type* type) {
    switch (type->kind) {
    case TYPE_KIND_UNRESOLVED:
        return format_string("UnresolvedType ('%s')", ((UnresolvedType*)type)->name);

    case TYPE_KIND_VALUE:
        return format_string("%s", value_type_kind_to_string(((ValueType*)type)->value_kind));
    }
}

void type_destroy(Type* type) {
    switch (type->kind) {
    case TYPE_KIND_UNRESOLVED:
        auto unresolved_type = (UnresolvedType*)type;
        free(unresolved_type->name);

        break;

    case TYPE_KIND_VALUE:
        break;
    }

    free(type);
}
