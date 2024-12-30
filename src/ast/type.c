#include "type.h"
#include "../string/format_string.h"
#include "type-kind.h"
#include <stdlib.h>
#include <string.h>

UnresolvedType* type_create_unresolved(bool is_pointer, char* name) {
    UnresolvedType* type = malloc(sizeof(UnresolvedType));
    if (!type) {
        return 0;
    }

    type->is_resolved = false;
    type->is_pointer = is_pointer;
    type->name = strdup(name);

    return type;
}

ResolvedType* type_create_resolved(bool is_pointer, TypeKind kind) {
    ResolvedType* type = malloc(sizeof(ResolvedType));
    if (!type) {
        return 0;
    }

    type->is_resolved = true;
    type->is_pointer = is_pointer;
    type->kind = kind;

    return type;
}

bool type_equal(Type* type_a, Type* type_b) {
    // If one type is a pointer, and the other isn't, they do not match.
    if (type_a->is_pointer != type_b->is_pointer) {
        return false;
    }

    // If one type is resolved, and the other isn't, they do not match.
    if (type_a->is_resolved != type_b->is_resolved) {
        return false;
    }

    if (type_a->is_resolved) {
        ResolvedType* resolved_a = (ResolvedType*)type_a;
        ResolvedType* resolved_b = (ResolvedType*)type_b;

        // The kinds must match if the types are resolved.
        return resolved_a->kind == resolved_b->kind;
    } else {
        UnresolvedType* unresolved_a = (UnresolvedType*)type_a;
        UnresolvedType* unresolved_b = (UnresolvedType*)type_b;

        // The names must match if the types are unresolved.
        return strcmp(unresolved_a->name, unresolved_b->name) == 0;
    }
}

char* type_to_string(Type* type) {
    if (type->is_resolved) {
        ResolvedType* resolved_type = (ResolvedType*)type;
        if (resolved_type->is_pointer) {
            return format_string("*%s", type_kind_to_string(resolved_type->kind));
        } else {
            return type_kind_to_string(resolved_type->kind);
        }
    } else {
        UnresolvedType* unresolved_type = (UnresolvedType*)type;
        return format_string("unresolved ('*%s')", unresolved_type->name);
    }
}

void type_destroy(Type* type) {
    // If this is an unresolved type, we can free its name.
    if (!type->is_resolved) {
        UnresolvedType* unresolved = (UnresolvedType*)type;
        free(unresolved->name);
    }

    // Free the type itself.
    // free(type);
}
