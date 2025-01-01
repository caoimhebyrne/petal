#include "vector.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

Vector* vector_create(size_t capacity) {
    Vector* vector = malloc(sizeof(Vector));
    if (!vector) {
        fprintf(stderr, "fatal error: failed to allocate vector");
        return 0;
    }

    void** items = malloc(sizeof(*vector->items) * capacity);
    if (!items) {
        fprintf(stderr, "fatal error: failed to allocate array of capacity %zu for vector", capacity);
        return 0;
    }

    vector->items = items;
    vector->size = 0;
    vector->capacity = capacity;

    return vector;
}

// Attempts to resize the vector to a new capacity.
bool vector_resize(Vector* vector, size_t capacity) {
    void** items = realloc(vector->items, sizeof(*vector->items) * capacity);
    if (!items) {
        fprintf(stderr, "fatal error: failed to allocate array of capacity %zu for vector", capacity);
        return false;
    }

    // The new items must be initialized to a null pointer to prevent mangling.
    for (size_t i = vector->capacity; i < capacity; i++) {
        items[i] = 0;
    }

    vector->items = items;
    vector->capacity = capacity;

    return true;
}

bool vector_append(Vector* vector, void* item, size_t item_size) {
    // If there is not enough capacity, we must resize the vector.
    if (vector->size >= vector->capacity) {
        if (!vector_resize(vector, vector->capacity * 2)) {
            return false;
        }
    }

    void* copy = malloc(item_size);
    if (!copy) {
        fprintf(stderr, "fatal error: failed to allocate %zu bytes for item copy", item_size);
        return false;
    }

    memcpy(copy, item, item_size);
    vector->items[vector->size++] = copy;

    return true;
}

void vector_destroy(Vector* vector) {
    for (size_t i = 0; i < vector->size; i++) {
        free(vector->items[i]);
    }

    free(vector->items);
    free(vector);
}
