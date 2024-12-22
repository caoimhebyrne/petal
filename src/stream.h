#ifndef __STREAM_H__
#define __STREAM_H__

#include "logger.h"
#include <stdbool.h>
#include <stddef.h>
#include <stdlib.h>
#include <string.h>

#define DECLARE_STREAM(name, lowercase_name, value_type)                                                               \
    typedef struct {                                                                                                   \
        value_type* data;                                                                                              \
        size_t length;                                                                                                 \
        size_t capacity;                                                                                               \
    } name;                                                                                                            \
                                                                                                                       \
    bool lowercase_name##_initialize(name* stream, size_t initial_capacity);                                           \
    bool lowercase_name##_append(name* stream, value_type value);                                                      \
    void lowercase_name##_destroy(name* stream);

#define CREATE_STREAM(name, lowercase_name, value_type)                                                                \
    bool lowercase_name##_initialize(name* stream, size_t initial_capacity) {                                          \
        value_type* data = malloc(sizeof(value_type) * initial_capacity);                                              \
        if (!data) {                                                                                                   \
            LOG_ERROR("stream", "failed to allocate stream buffer of %zu", initial_capacity);                          \
            return false;                                                                                              \
        }                                                                                                              \
                                                                                                                       \
        /* Ensure that all of the data within the allocated buffer is zero'd out. */                                   \
        memset(data, 0, sizeof(value_type) * initial_capacity);                                                        \
                                                                                                                       \
        stream->data = data;                                                                                           \
        stream->length = 0;                                                                                            \
        stream->capacity = initial_capacity;                                                                           \
                                                                                                                       \
        return true;                                                                                                   \
    }                                                                                                                  \
                                                                                                                       \
    bool lowercase_name##_append(name* stream, value_type value) {                                                     \
        if (stream->length >= stream->capacity) {                                                                      \
            size_t new_capacity = stream->capacity * 2;                                                                \
                                                                                                                       \
            value_type* new_data = realloc(stream->data, sizeof(value_type) * new_capacity);                           \
            if (!new_data) {                                                                                           \
                LOG_ERROR("stream", "failed to re-allocated stream buffer from %zu to %zu", stream->capacity,          \
                          new_capacity);                                                                               \
                return false;                                                                                          \
            }                                                                                                          \
                                                                                                                       \
            /* Ensure that all of the newly allocated cells are initialized to zero. */                                \
            for (size_t i = stream->capacity; i < new_capacity; i++) {                                                 \
                new_data[i] = (value_type){};                                                                          \
            }                                                                                                          \
                                                                                                                       \
            stream->data = new_data;                                                                                   \
            stream->capacity = new_capacity;                                                                           \
        }                                                                                                              \
                                                                                                                       \
        stream->data[stream->length++] = value;                                                                        \
        return true;                                                                                                   \
    }

#endif
