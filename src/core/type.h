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

// An unresolved type, the AST produces this for the typechecker to resolve and validate.
typedef struct {
    union {
        Type header;
    };

    // The name of the type, for example: "i32".
    char* name;
} UnresolvedType;

// Represents the different kinds of value types.
typedef enum {
    // An invalid value type.
    VALUE_TYPE_KIND_INVALID,

    // A signed 32-bit integer.
    VALUE_TYPE_KIND_I32,

    // A 64-bit floating point.
    VALUE_TYPE_KIND_F64,
} ValueTypeKind;

// Returns a value type kind from a string.
// If unsuccessful, VALUE_TYPE_KIND_INVALID is returned.
ValueTypeKind value_type_kind_from_string(char* value);

// Returns a string representation of a value type kind.
const char* value_type_kind_to_string(ValueTypeKind kind);

// A simple value type, for example: a signed 32-bit integer.
typedef struct {
    union {
        Type header;
    };

    // The kind of the value type.
    ValueTypeKind value_kind;
} ValueType;

// Creates a new unresolved type.
// Parameters:
// - position: The position that this file occurred at within the source file.
// - name: The name of the type.
// Returns: A reference to an `UnresolvedType` if successful, otherwise nullptr.
UnresolvedType* type_create_unresolved(Position position, char* name);

// Creates a new ValueType.
// Parameters:
// - position: The position that this file occurred at within the source file.
// - kind: The kind of value type that this is.
// Returns: A reference to an `ValueType` if successful, otherwise nullptr.
ValueType* type_create_value(Position position, ValueTypeKind kind);

// Returns a heap-allocated string represenatation of a Type.
char* type_to_string(Type* type);

// De-allocates a Type.
// Parameters:
// - type: The type to destroy.
void type_destroy(Type* type);
