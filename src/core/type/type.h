#pragma once

#include "core/position.h"

// Represents the different kinds of types.
typedef enum {
    // An unresolved type, just holds a string for the type's name.
    TYPE_KIND_UNRESOLVED,

    // A value type.
    TYPE_KIND_VALUE,
} TypeKind;

// Represents a standard type.
typedef struct {
    // The kind of type that this is.
    TypeKind kind;

    // The position that this type occured at within the source file.
    Position position;
} Type;

// Returns a heap-allocated string represenatation of a Type.
char* type_to_string(Type* type);

// De-allocates a Type.
// Parameters:
// - type: The type to destroy.
void type_destroy(Type* type);
