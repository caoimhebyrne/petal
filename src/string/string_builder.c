#include "string_builder.h"
#include <stdlib.h>

CREATE_STREAM(StringBuilder, string_builder, char)

char* string_builder_finish(StringBuilder* builder) {
    char* buffer = realloc(builder->data, builder->length + 1);
    buffer[builder->length] = '\0';

    return buffer;
}
