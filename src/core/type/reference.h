#pragma once

#include "core/position.h"
#include "core/type/type.h"

// A type that should be considered as a reference to another type.
typedef struct {
    union {
        Type header;
    };

    // The type being treated as a reference.
    Type* referenced_type;
} ReferenceType;

// Creates a new ReferenceType.
// Parameters:
// - position: The position that this type occurred at within the source file.
// - referenced_type: The type being referenced.
ReferenceType* reference_type_create(Position position, Type* referenced_type);
