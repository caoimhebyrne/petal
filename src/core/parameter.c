#include "core/parameter.h"
#include "util/defer.h"
#include "util/format.h"
#include <stdlib.h>

Parameter parameter_create(Position position, char* name, Type* value_type) {
    return (Parameter){
        .position = position,
        .name = name,
        .value_type = value_type,
    };
}

char* parameter_to_string(Parameter parameter) {
    auto value_type_string defer(free_str) = type_to_string(parameter.value_type);
    return format_string("Parameter { name = '%s', type = %s }", parameter.name, value_type_string);
}

void parameter_destroy(Parameter parameter) {
    type_destroy(parameter.value_type);
    free(parameter.name);
}
