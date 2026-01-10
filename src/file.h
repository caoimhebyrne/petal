#pragma once

#include "array.h"
#include <stdbool.h>

#ifdef _WIN32
#define PATH_SEPARATOR '\\'
#else
#define PATH_SEPARATOR '/'
#endif // _WIN32

/**
 * Attempts to read a file at [path] into a StringBuffer.
 * Returns false if the file could not be read, an error reason will be written to stderr.
 */
bool file_read(const char* path, StringBuffer* output);
