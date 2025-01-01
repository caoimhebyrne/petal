#ifndef __UTIL_STRING_BUILDER_H__
#define __UTIL_STRING_BUILDER_H__

#include "util/vector.h"
#include <stdbool.h>
#include <stddef.h>

typedef struct {
    // A vector of characters being built by this string builder.
    Vector(char) vector;
} StringBuilder;

// Initializes a new string builder.
// Use `string_builder_is_invalid` to check for failure.
// Returns:
// - A valid string builder reference if successful.
// - A string builder with a capacity of zero if an error occurred during allocation.
StringBuilder string_builder_create();

// Returns whether the provided StringBuilder is invalid.
bool string_builder_is_invalid(StringBuilder builder);

// Returns the amount of characters stored in this builder
size_t string_builder_length(StringBuilder builder);

// Appends a character to a string builder.
// Returns whether the operation was successful.
bool string_builder_append(StringBuilder* builder, char character);

// Attempts to finalize this string builder.
// This also calls `string_builder_destroy`.
// Returns:
// - If successful: the string held by the string builder, followed by a null byte.
// - A null pointer if an error occurred.
char* string_builder_finish(StringBuilder* builder);

// Destroys a StringBuilder and its contents.
// If `string_builder_finish` was called, the contents returned by that function will not be de-allocated.
void string_builder_destroy(StringBuilder* builder);

#endif // __UTIL_STRING_BUILDER_H__
