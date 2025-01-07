#pragma once

#include "core/type/type.h"
#include "util/vector.h"

// A declared variable to be recorded by the typechecker.
typedef struct {
    // The name of this variable.
    char* name;

    // The type of this variable.
    Type* type;
} DeclaredVariable;

typedef Vector(DeclaredVariable) DeclaredVariableVector;

// Creates a new DeclaredVariable.
// Parameters:
// - name: The name of this variable.
// - type: The type of this variable.
DeclaredVariable declared_variable_create(char* name, Type* type);

// Finds a declared variable by its name.
// Parameters:
// - name: The name of the variable.
// Returns: A reference to a declared variable if it exists, otherwise nullptr.
DeclaredVariable* declared_variable_find_by_name(DeclaredVariableVector variables, char* name);
