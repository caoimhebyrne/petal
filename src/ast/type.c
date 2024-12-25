#include "type.h"
#include <string.h>

Type type_create(TypeKind kind, bool is_pointer) { return (Type){kind, is_pointer}; }

TypeKind type_kind_from_string(char* value) {
    if (strcmp(value, "void") == 0) {
        return TYPE_KIND_VOID;
    } else if (strcmp(value, "i32") == 0) {
        return TYPE_KIND_INT_32;
    } else if (strcmp(value, "i64") == 0) {
        return TYPE_KIND_INT_64;
    } else if (strcmp(value, "f32") == 0) {
        return TYPE_KIND_FLOAT_32;
    } else {
        return TYPE_KIND_INVALID;
    }
}

char* type_to_string(Type type) {
    switch (type.kind) {
    case TYPE_KIND_INVALID:
        return "invalid type";

    case TYPE_KIND_INT_32:
        return "i32";

    case TYPE_KIND_INT_64:
        return "i64";

    case TYPE_KIND_FLOAT_32:
        return "f32";

    case TYPE_KIND_VOID:
        return "void";
    }

    // This should never happen, the compiler is being weird.
    return "unknown type";
}
