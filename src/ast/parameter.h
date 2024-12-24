#ifndef __PARAMETER_H__
#define __PARAMETER_H__

#include "../stream.h"
#include "type.h"

typedef struct {
    // The name of this parameter.
    char* name;

    // The type of this parameter.
    Type type;
} Parameter;

// Creates a new Parameter with a name and type.
// Parameters:
// - name: The name of this parameter.
// - type: The type of this parameter.
Parameter parameter_create(char* name, Type type);

DECLARE_STREAM(Parameters, parameters, Parameter)

#endif
