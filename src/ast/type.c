#include "type.h"
#include <string.h>

Type type_from_string(char* value) {
    if (strcmp(value, "void") == 0) {
        return TYPE_VOID;
    } else if (strcmp(value, "i32") == 0) {
        return TYPE_INT_32;
    } else if (strcmp(value, "i64") == 0) {
        return TYPE_INT_64;
    } else if (strcmp(value, "f32") == 0) {
        return TYPE_FLOAT_32;
    } else {
        return TYPE_INVALID;
    }
}

char* type_to_string(Type type) {
    switch (type) {
    case TYPE_INVALID:
        return "invalid type";

    case TYPE_INT_32:
        return "i32";

    case TYPE_INT_64:
        return "i64";

    case TYPE_FLOAT_32:
        return "f32";

    case TYPE_VOID:
        return "void";
    }

    // This should never happen, the compiler is being weird.
    return "unknown type";
}
