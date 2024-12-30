#include "type-kind.h"
#include <string.h>

TypeKind type_kind_from_string(char* value) {
    if (strcmp(value, "void") == 0) {
        return TYPE_KIND_VOID;
    } else if (strcmp(value, "bool") == 0) {
        return TYPE_KIND_BOOL;
    } else if (strcmp(value, "i8") == 0) {
        return TYPE_KIND_INT_8;
    } else if (strcmp(value, "i32") == 0) {
        return TYPE_KIND_INT_32;
    } else if (strcmp(value, "i64") == 0) {
        return TYPE_KIND_INT_64;
    } else if (strcmp(value, "f32") == 0) {
        return TYPE_KIND_FLOAT_32;
    } else if (strcmp(value, "f64") == 0) {
        return TYPE_KIND_FLOAT_64;
    } else {
        return TYPE_KIND_INVALID;
    }
}

// Returns a string represenation of the provided type kind.
char* type_kind_to_string(TypeKind kind) {
    switch (kind) {
    case TYPE_KIND_VOID:
        return "void";

    case TYPE_KIND_BOOL:
        return "bool";

    case TYPE_KIND_INT_8:
        return "i8";

    case TYPE_KIND_INT_32:
        return "i32";

    case TYPE_KIND_INT_64:
        return "i64";

    case TYPE_KIND_FLOAT_32:
        return "f32";

    case TYPE_KIND_FLOAT_64:
        return "f64";

    case TYPE_KIND_INVALID:
        return "invalid type";
    }
}
