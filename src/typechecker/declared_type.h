#pragma once

#include "core/type/type.h"
#include "util/vector.h"

typedef struct {
    // The name of the type being declared.
    char* name;

    // The type itself.
    Type* type;
} DeclaredType;

typedef Vector(DeclaredType) DeclaredTypeVector;

// Creates a new DeclaredType.
// Parameters:
// - name: The name of the type being declared.
// - type: The type itself.
DeclaredType declared_type_create(char* name, Type* type);

// Finds a declared type by its name.
// Parameters:
// - name: The name of the type.
// Returns: A reference to a declared type if it exists, otherwise nullptr.
DeclaredType* declared_type_find_by_name(DeclaredTypeVector types, char* name);
