#include "type.h"
#include "core/type/reference.h"
#include "core/type/unresolved.h"
#include "core/type/value.h"
#include "util/defer.h"
#include "util/format.h"
#include <stdlib.h>
#include <string.h>

bool type_equals(Type* left, Type* right) {
    // If the two types are not the same kind, they are not compatible.
    if (left->kind != right->kind) {
        return false;
    }

    // We can further check for equality based on the type's kind.
    switch (left->kind) {
    case TYPE_KIND_UNRESOLVED:
        // For unresolved types, their names must be equal.
        return strcmp(((UnresolvedType*)left)->name, ((UnresolvedType*)right)->name) == 0;

    case TYPE_KIND_VALUE:
        // For value types, their value kinds must match.
        return ((ValueType*)left)->value_kind == ((ValueType*)right)->value_kind;

    case TYPE_KIND_REFERENCE:
        // For reference types, their referenced types must match.
        return type_equals(((ReferenceType*)left)->referenced_type, ((ReferenceType*)right)->referenced_type);
    }
}

char* type_to_string(Type* type) {
    switch (type->kind) {
    case TYPE_KIND_UNRESOLVED:
        return format_string("UnresolvedType ('%s')", ((UnresolvedType*)type)->name);

    case TYPE_KIND_VALUE:
        return format_string("%s", value_type_kind_to_string(((ValueType*)type)->value_kind));

    case TYPE_KIND_REFERENCE:
        auto reference_type = (ReferenceType*)type;
        auto type_string defer(free_str) = type_to_string(reference_type->referenced_type);

        return format_string("&%s", type_string);
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

    case TYPE_KIND_REFERENCE:
        auto reference_type = (ReferenceType*)type;
        type_destroy(reference_type->referenced_type);

        break;
    }

    free(type);
}
