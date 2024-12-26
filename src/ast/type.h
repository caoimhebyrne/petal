#ifndef __TYPE_H__
#define __TYPE_H__

#include <stdbool.h>

typedef enum {
    // An invalid type.
    TYPE_KIND_INVALID,

    // void
    TYPE_KIND_VOID,

    // A 8-bit integer, defined as 'i8'.
    TYPE_KIND_INT_8,

    // A 32-bit integer, defined as 'i32'.
    TYPE_KIND_INT_32,

    // A 64-bit integer, defined as 'i64'.
    TYPE_KIND_INT_64,

    // A 32-bit float, defined as 'f32'.
    TYPE_KIND_FLOAT_32,
} TypeKind;

typedef struct {
    // The kind of value that this type holds.
    TypeKind kind;

    // Whether this is a pointer or not.
    bool is_pointer;
} Type;

#define TYPE_INVALID (Type){.kind = TYPE_KIND_INVALID, .is_pointer = false};

// Creates a Type with a certain kind.
// Parameters:
// - kind: The kind of value that this type holds.
// - is_pointer: Whether this is a pointer or not.
Type type_create(TypeKind kind, bool is_pointer);

// Returns a human-readable string representation of the provided Type.
// Parameters:
// - type: The type to stringify.
char* type_to_string(Type type);

// Returns a TypeKind for the provided string.
// Parameters:
// - value: The name of the type kind.
// Returns:
// A Type if the name matches an available type kind, otherwise TYPE_INVALID.
TypeKind type_kind_from_string(char* value);

#endif // __TYPE_H__
