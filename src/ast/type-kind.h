#ifndef __TYPE_KIND_H__
#define __TYPE_KIND_H__

typedef enum {
    // An invalid type.
    TYPE_KIND_INVALID,

    // void
    TYPE_KIND_VOID,

    // A boolean, defined as 'bool'.
    TYPE_KIND_BOOL,

    // A 8-bit integer, defined as 'i8'.
    TYPE_KIND_INT_8,

    // A 32-bit integer, defined as 'i32'.
    TYPE_KIND_INT_32,

    // A 64-bit integer, defined as 'i64'.
    TYPE_KIND_INT_64,

    // A 32-bit float, defined as 'f32'.
    TYPE_KIND_FLOAT_32,

    // A 64-bit float, defined as 'f64'.
    TYPE_KIND_FLOAT_64,
} TypeKind;

// Returns a TypeKind for the provided string.
// Parameters:
// - value: The name of the type kind.
// Returns:
// A Type if the name matches an available type kind, otherwise TYPE_KIND_INVALID.
TypeKind type_kind_from_string(char* value);

// Returns a string represenation of the provided type kind.
char* type_kind_to_string(TypeKind kind);

#endif // __TYPE_KIND_H__
