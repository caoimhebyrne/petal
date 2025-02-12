#pragma once

#include "util/vector.h"
#include <stddef.h>

typedef Vector(char*) StringVector;

typedef struct {
    // The data read from the file.
    char* data;

    // The length of the contents in bytes.
    size_t length;
} FileContents;

// Attempts to read the contents of a file at the provided path.
// The caller must call file_contents_destroy when finished reading from the data.
// Parameters:
// - path: The path to the file to read the contents of.
// Returns:
// - In all cases, a valid FileContents struct is returned.
//   If the read operation fails, the `contents` pointer will be a null-pointer.
FileContents file_read(char* path);

// Creates a vector of lines from a file's contents.
// Parameters:
// - contents: The file contents to read from.
StringVector file_contents_lines(FileContents contents);

// De-allocates the data read from a file.
void file_contents_destroy(FileContents contents);
