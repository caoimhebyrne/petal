#include "array.h"

IMPLEMENT_ARRAY_TYPE(StringBuffer, string_buffer, char)

bool string_buffer_equals(const StringBuffer* buffer, const StringBuffer* other) {
    if (buffer->length != other->length) {
        return false;
    }

    for (size_t i = 0; i < buffer->length; i++) {
        if (buffer->data[i] != other->data[i]) {
            return false;
        }
    }

    return true;
}
