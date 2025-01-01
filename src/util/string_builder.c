#include "string_builder.h"
#include "util/vector.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

StringBuilder string_builder_create() {
    Vector* vector = vector_create(2);
    if (!vector) {
        return (StringBuilder){0};
    }

    return (StringBuilder){vector};
}

bool string_builder_is_invalid(StringBuilder builder) {
    return !builder.vector;
}

size_t string_builder_length(StringBuilder builder) {
    if (!string_builder_is_invalid(builder)) {
        return 0;
    }

    return builder.vector->size;
}

bool string_builder_append(StringBuilder* builder, char character) {
    return vector_append(builder->vector, &character, sizeof(char));
}

char* string_builder_finish(StringBuilder* builder) {
    char* buffer = malloc(builder->vector->size + 1);
    if (!buffer) {
        fprintf(stderr, "fatal error: failed to allocate %zu bytes for string", builder->vector->size + 1);
        return 0;
    }

    for (size_t i = 0; i < builder->vector->size; i++) {
        char* item = builder->vector->items[i];
        buffer[i] = *item;
    }

    // Set the last character to a null byte.
    buffer[builder->vector->size] = '\0';

    string_builder_destroy(builder);
    return buffer;
}

void string_builder_destroy(StringBuilder* builder) {

    vector_destroy(builder->vector);
}
