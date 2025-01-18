#pragma once

#include "core/parameter.h"
#include "core/type/type.h"
#include "util/vector.h"

// A declared function to be recorded by the typechecker.
typedef struct {
    // The name of this function.
    char* name;

    // The return type of this function.
    Type* return_type;

    // The parameters of this function.
    ParameterVector* parameters;
} DeclaredFunction;

typedef Vector(DeclaredFunction) DeclaredFunctionVector;

// Creates a new DeclaredFunction.
// Parameters:
// - name: The name of this function.
// - return_type: The return type of this function.
// - parameters: The parameters of this function.
DeclaredFunction declared_function_create(char* name, Type* type, ParameterVector* parameters);

// Finds a declared variable by its name.
// Parameters:
// - name: The name of the function.
// Returns: A reference to a declared variable if it exists, otherwise nullptr.
DeclaredFunction* declared_function_find_by_name(DeclaredFunctionVector functions, char* name);
