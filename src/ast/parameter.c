#include "parameter.h"

Parameter parameter_create(char* name, Type type) { return (Parameter){name, type}; }

CREATE_STREAM(Parameters, parameters, Parameter);
