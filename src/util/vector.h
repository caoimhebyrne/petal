#pragma once

#include "util/logger.h"
#include <stdbool.h>
#include <stddef.h>
#include <stdio.h>  // IWYU pragma: keep
#include <stdlib.h> // IWYU pragma: keep

// Whether vector resizes should be logged.
extern bool log_vector_resizing;

// Defines a new vector which has elements of type T.
#define Vector(T)                                                                                                      \
    struct {                                                                                                           \
        T* items;                                                                                                      \
        size_t length;                                                                                                 \
        size_t capacity;                                                                                               \
    }

// Returns a new Vector with a capacity of zero.
#define vector_create() {0}

// Returns the size of one item in the vector.
#define vector_item_size(vector) sizeof(*(vector).items)

// Returns the item at a certain index in the vector.
#define vector_get(vector, i) ((vector))->items[i]

// Returns a reference to the item at a certain index in the vector.
#define vector_get_ref(vector, i) &((vector))->items[i]

// Returns the last item in a vector.
#define vector_last(vector) (vector).items[(vector).length - 1]

// Initializes a vector with a certain capacity.
#define vector_initialize(vector, initial_capacity)                                                                    \
    ({                                                                                                                 \
        bool success = true;                                                                                           \
                                                                                                                       \
        vector.items = malloc(initial_capacity * vector_item_size(vector));                                            \
        if (!vector.items) {                                                                                           \
            fprintf(                                                                                                   \
                stderr,                                                                                                \
                "failed to initialize vector with capacity of %d (item size: %zu)\n",                                  \
                initial_capacity,                                                                                      \
                vector_item_size(vector)                                                                               \
            );                                                                                                         \
                                                                                                                       \
            success = false;                                                                                           \
        }                                                                                                              \
                                                                                                                       \
        if (log_vector_resizing) {                                                                                     \
            LOG_DEBUG(                                                                                                 \
                "vector",                                                                                              \
                "(%s:%d) initialized vector with capacity of %d",                                                      \
                __FILE__,                                                                                              \
                __LINE__,                                                                                              \
                initial_capacity                                                                                       \
            );                                                                                                         \
        }                                                                                                              \
                                                                                                                       \
        vector.capacity = initial_capacity;                                                                            \
        success;                                                                                                       \
    })

#define vector_resize(vector, new_capacity)                                                                            \
    ({                                                                                                                 \
        bool success = true;                                                                                           \
                                                                                                                       \
        ((vector))->items = realloc(((vector))->items, new_capacity * vector_item_size(*((vector))));                  \
        if (((vector))->items) {                                                                                       \
            if (log_vector_resizing) {                                                                                 \
                LOG_DEBUG(                                                                                             \
                    "vector",                                                                                          \
                    "(%s:%d) resized vector from capacity of %zu to %zu",                                              \
                    __FILE__,                                                                                          \
                    __LINE__,                                                                                          \
                    ((vector))->capacity,                                                                              \
                    new_capacity                                                                                       \
                );                                                                                                     \
            }                                                                                                          \
                                                                                                                       \
            ((vector))->capacity = new_capacity;                                                                       \
        } else {                                                                                                       \
            fprintf(stderr, "failed to resize vector to %zu!\n", new_capacity* vector_item_size(*((vector))));         \
            success = false;                                                                                           \
        }                                                                                                              \
                                                                                                                       \
        success;                                                                                                       \
    })

#define vector_append(vector, item)                                                                                    \
    ({                                                                                                                 \
        bool success = true;                                                                                           \
                                                                                                                       \
        if (((vector))->length >= ((vector))->capacity) {                                                              \
            success = vector_resize(((vector)), ((vector))->capacity * 2);                                             \
        }                                                                                                              \
                                                                                                                       \
        if (success) {                                                                                                 \
            ((vector))->items[((vector))->length++] = item;                                                            \
        }                                                                                                              \
                                                                                                                       \
        success;                                                                                                       \
    })

#define vector_destroy(vector, destroy_function)                                                                       \
    ({                                                                                                                 \
        for (size_t i = 0; i < vector.length; i++) {                                                                   \
            if (destroy_function != NULL) {                                                                            \
                destroy_function(vector.items[i]);                                                                     \
            }                                                                                                          \
        }                                                                                                              \
                                                                                                                       \
        free(vector.items);                                                                                            \
    })
