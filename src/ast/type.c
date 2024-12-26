#include "type.h"
#include "../string/format_string.h"
#include <string.h>

Type type_create(TypeKind kind, bool is_pointer) { return (Type){kind, is_pointer}; }

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
    } else {
        return TYPE_KIND_INVALID;
    }
}

char* type_to_string(Type type) {
    char* type_name = "unknown type";

    switch (type.kind) {
    case TYPE_KIND_INVALID:
        type_name = "invalid type";
        break;

    case TYPE_KIND_BOOL:
        type_name = "bool";
        break;

    case TYPE_KIND_INT_8:
        type_name = "i8";
        break;

    case TYPE_KIND_INT_32:
        type_name = "i32";
        break;

    case TYPE_KIND_INT_64:
        type_name = "i64";
        break;

    case TYPE_KIND_FLOAT_32:
        type_name = "f32";
        break;

    case TYPE_KIND_VOID:
        type_name = "void";
        break;
    }

    if (type.is_pointer) {
        return format_string("*%s", type_name);
    }

    return type_name;
}
