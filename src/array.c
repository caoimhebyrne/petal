#include "array.h"
#include "allocator.h"
#include "assert.h"
#include <stdarg.h>
#include <stdio.h>
#include <string.h>

IMPLEMENT_ARRAY_TYPE(StringBuffer, string_buffer, char)

void string_buffer_init_from_cstr(StringBuffer* buffer, Allocator* allocator, const char* cstr) {
    string_buffer_init(buffer, allocator);
    // FIXME: This is a bad cast, but the value being `const` seems to cause issues with pointer types.
    string_buffer_append_many(buffer, (char*)cstr, strlen(cstr));
}

void string_buffer_init_fmt(StringBuffer* buffer, Allocator* allocator, const char* format, ...) {
    va_list args;
    va_start(args, format);

    string_buffer_init_vfmt(buffer, allocator, format, args);

    va_end(args);
}

void string_buffer_init_vfmt(StringBuffer* buffer, Allocator* allocator, const char* format, va_list args) {
    assert(buffer != NULL && "NULL buffer passed to string_buffer_init_fmt");

    va_list args_copy;
    va_copy(args_copy, args);

    const size_t string_length = vsnprintf(NULL, 0, format, args);

    assert(string_length > 0 && "Failed to init StringBuffer with format string (vsnprintf failed)");

    string_buffer_init(buffer, allocator);
    string_buffer_resize(buffer, sizeof(char) * string_length);

    // We add 1 to the length because vsnprintf assumes that the provided pointer is null-terminated.
    vsnprintf(buffer->data, string_length + 1, format, args_copy);
    va_end(args_copy);

    buffer->length = string_length;
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
