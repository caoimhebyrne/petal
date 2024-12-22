#include "string_builder.h"
#include <stdlib.h>
#include <string.h>

CREATE_STREAM(StringBuilder, string_builder, char)

bool string_builder_append_string(StringBuilder* builder, char* string) {
    for (size_t i = 0; i < strlen(string); i++) {
        if (!string_builder_append(builder, string[i])) {
            return false;
        }
    }

    return true;
}

char* string_builder_finish(StringBuilder* builder) {
    char* buffer = realloc(builder->data, builder->length + 1);
    buffer[builder->length] = '\0';

    return buffer;
}
