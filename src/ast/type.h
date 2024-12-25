#ifndef __TYPE_H__
#define __TYPE_H__

typedef enum {
    // An invalid type.
    TYPE_INVALID,

    // void
    TYPE_VOID,

    // A 32-bit integer, defined as 'i32'.
    TYPE_INT_32,

    // A 64-bit integer, defined as 'i64'.
    TYPE_INT_64,

    // A 32-bit float, defined as 'f32'.
    TYPE_FLOAT_32,
} Type;

// Returns a Type for the provided v string.
// Parameters:
// - value: The name of the type.
// Returns:
// A Type if the name matches an available type, otherwise TYPE_INVALID.
Type type_from_string(char* value);

// Returns a human-readable string representation of the provided Type.
// Parameters:
// - type: The type to stringify.
char* type_to_string(Type type);

#endif // __TYPE_H__
