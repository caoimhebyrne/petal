#include "array.h"
#include "allocator.h"
#include "logger.h"
#include <string.h>

IMPLEMENT_ARRAY_TYPE(StringBuffer, string_buffer, char)

void string_buffer_init_from_cstr(StringBuffer* buffer, Allocator* allocator, const char* cstr) {
    string_buffer_init(buffer, allocator);
    // FIXME: This is a bad cast, but the value being `const` seems to cause issues with pointer types.
    string_buffer_append_many(buffer, (char*)cstr, strlen(cstr));
}

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

bool string_buffer_equals_cstr(const StringBuffer* buffer, const char* cstr) {
    if (buffer->length != strlen(cstr)) {
        return false;
    }

    for (size_t i = 0; i < buffer->length; i++) {
        if (buffer->data[i] != cstr[i]) {
            return false;
        }
    }

    return true;
}

void string_buffer_trim_from(StringBuffer* buffer, const char character) {
    size_t character_index = 0;

    // We need to get the index of the last occurrence of the character.
    for (size_t i = 0; i < buffer->length; i++) {
        if (character == buffer->data[i]) {
            character_index = i;
        }
    }

    // If the character index is still 0, then we could not find an occurrence of the provided character.
    if (character_index == 0) {
        return;
    }

    // Then, we can set the StringBuffer's length and data accordingly.
    buffer->length = buffer->length - (character_index + 1);
    buffer->data = buffer->data + (character_index + 1);
}

void string_buffer_trim_until(StringBuffer* buffer, const char character) {
    size_t character_index = 0;

    // We need to get the index of the last occurrence of the character.
    for (size_t i = 0; i < buffer->length; i++) {
        if (character == buffer->data[i]) {
            character_index = i;
            break;
        }
    }

    // If the character index is still 0, then we could not find an occurrence of the provided character.
    if (character_index == 0) {
        return;
    }

    // Then, we can set the StringBuffer's length to the position before the character.
    buffer->length = character_index;
}
