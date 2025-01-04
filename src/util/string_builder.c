#include "string_builder.h"
#include "util/vector.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

StringBuilder string_builder_create() {
    StringBuilder builder = {.vector = vector_create()};
    if (!vector_initialize(builder.vector, 2)) {
        return (StringBuilder){};
    }

    return builder;
}

bool string_builder_is_invalid(StringBuilder builder) {
    return builder.vector.capacity == 0;
}

size_t string_builder_length(StringBuilder builder) {
    if (string_builder_is_invalid(builder)) {
        return 0;
    }

    return builder.vector.length;
}

bool string_builder_append(StringBuilder* builder, char character) {
    return vector_append(&builder->vector, character);
}

bool string_builder_append_str(StringBuilder* builder, const char* string) {
    for (size_t i = 0; i < strlen(string); i++) {
        auto character = string[i];
        if (character == '\0') {
            break;
        }

        if (!string_builder_append(builder, string[i])) {
            return false;
        }
    }

    return true;
}

typedef void (*func_ptr)(int);

char* string_builder_finish(StringBuilder* builder) {
    char* buffer = malloc(builder->vector.length + 1);
    if (!buffer) {
        fprintf(stderr, "fatal error: failed to allocate %zu bytes for string", builder->vector.length + 1);
        return nullptr;
    }

    for (size_t i = 0; i < builder->vector.length; i++) {
        auto item = builder->vector.items[i];
        buffer[i] = item;
    }

    // Set the last character to a null byte.
    buffer[builder->vector.length] = '\0';

    string_builder_destroy(builder);
    return buffer;
}

void string_builder_destroy(StringBuilder* builder) {
    free(builder->vector.items);
}
