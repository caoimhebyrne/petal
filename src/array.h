#pragma once

#include "allocator.h"
#include "math.h"
#include <stdbool.h>
#include <stddef.h>
#include <stdio.h>

// Dynamic arrays are used in a lot of places. This is an attempt to implement a generic solution without having
// access to generic type parameters.

// This should be called within a header to declare the type of an array.
#define DEFINE_ARRAY_TYPE(name, lower_name, T)                                                                         \
    typedef struct {                                                                                                   \
        Allocator *allocator;                                                                                          \
        T *data;                                                                                                       \
        size_t length;                                                                                                 \
        size_t capacity;                                                                                               \
    } name;                                                                                                            \
    void lower_name##_init(name *lower_name, Allocator *allocator);                                                    \
    void lower_name##_resize(name *lower_name, const size_t new_size);                                                 \
    void lower_name##_append(name *lower_name, const T value);                                                         \
    void lower_name##_append_many(name *lower_name, const T *values, const size_t size);

// This should be called within a `.c` file to provide implementations for methods belonging to an array type.
#define IMPLEMENT_ARRAY_TYPE(name, lower_name, T)                                                                      \
    void lower_name##_init(name *lower_name, Allocator *allocator) {                                                   \
        (lower_name)->allocator = allocator;                                                                           \
        (lower_name)->data = NULL;                                                                                     \
        (lower_name)->length = 0;                                                                                      \
        (lower_name)->capacity = 0;                                                                                    \
    }                                                                                                                  \
    void lower_name##_resize(name *lower_name, const size_t new_capacity) {                                            \
        (lower_name)->data = allocator_realloc((lower_name)->allocator, (lower_name)->data,                            \
                                               (lower_name)->capacity * sizeof(T), new_capacity * sizeof(T));          \
        (lower_name)->capacity = new_capacity;                                                                         \
    }                                                                                                                  \
    void lower_name##_append(name *lower_name, const T value) {                                                        \
        if ((lower_name)->length + 1 > (lower_name)->capacity) {                                                       \
            lower_name##_resize((lower_name), min(1, 2 * (lower_name)->capacity));                                     \
        }                                                                                                              \
        (lower_name)->data[(lower_name)->length++] = (value);                                                          \
    }                                                                                                                  \
    void lower_name##_append_many(name *lower_name, const T *values, const size_t size) {                              \
        for (size_t i = 0; i < size; i++) {                                                                            \
            lower_name##_append((lower_name), values[i]);                                                              \
        }                                                                                                              \
    }

// Declare some basic array types that are used throughout the project.
DEFINE_ARRAY_TYPE(StringBuffer, string_buffer, char)

// Checks whether the contents in two [StringBuffer]s are equal to each other.
bool string_buffer_equals(const StringBuffer *buffer, const StringBuffer *other);
