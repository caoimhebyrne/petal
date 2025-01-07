#pragma once

#include "core/type/type.h"

// Represents the different kinds of value types.
typedef enum {
    // An invalid value type.
    VALUE_TYPE_KIND_INVALID,

    // void.
    VALUE_TYPE_VOID,

    // A signed 32-bit integer.
    VALUE_TYPE_KIND_I32,

    // A 64-bit floating point.
    VALUE_TYPE_KIND_F64,
} ValueTypeKind;

// A simple value type, for example: a signed 32-bit integer.
typedef struct {
    union {
        Type header;
    };

    // The kind of the value type.
    ValueTypeKind value_kind;
} ValueType;

// Creates a new ValueType.
// Parameters:
// - position: The position that this file occurred at within the source file.
// - kind: The kind of value type that this is.
// Returns: A reference to an `ValueType` if successful, otherwise nullptr.
ValueType* value_type_create(Position position, ValueTypeKind kind);

// Returns a value type kind from a string.
// If unsuccessful, VALUE_TYPE_KIND_INVALID is returned.
ValueTypeKind value_type_kind_from_string(char* value);

// Returns a string representation of a value type kind.
const char* value_type_kind_to_string(ValueTypeKind kind);
