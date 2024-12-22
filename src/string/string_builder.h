#ifndef __STRING_BUILDER_H__
#define __STRING_BUILDER_H__

#include "../stream.h"
#include <stdbool.h>
#include <stddef.h>

DECLARE_STREAM(StringBuilder, string_builder, char);

// Appends a string of characters to the StringBuilder.
// Parameters:
// - string: The string to append.
bool string_builder_append_string(StringBuilder* builder, char* string);

// Finalizes the StringBuilder, returning the string contents.
// Parameters:
// - builder: The string builder to finalize.
// Return:
// - A string if successful, otherwise 0.
char* string_builder_finish(StringBuilder* builder);

#endif // __STRING_BUILDER_H__
