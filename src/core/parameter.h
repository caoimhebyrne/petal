#pragma once

#include "core/position.h"
#include "core/type/type.h"
#include "util/vector.h"

// A parameter defined in a function's declaration.
typedef struct {
    // The position that this paramater was defined at in the source file.
    Position position;

    // The name of this parameter.
    char* name;

    // The type of the value that this parameter expects.
    Type* value_type;
} Parameter;

// A vector of `Parameter`s.
typedef Vector(Parameter) ParameterVector;

// Creates a new parameter.
// Parameters:
// - name: The name of this parameter
// - value_type: The type of the value that this parameter expects.
Parameter parameter_create(Position position, char* name, Type* value_type);

// Returns a heap-allocated string representation of a Parameter.
char* parameter_to_string(Parameter parameter);

// De-allocates a Parameter's data.
// Parameters:
// - parameter: The parameter to destroy.
void parameter_destroy(Parameter parameter);
