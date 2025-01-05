#include "file.h"
#include "util/string_builder.h"
#include "util/vector.h"

#include <errno.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/stat.h>

FileContents file_read(char* path) {
    // Attempt to open the file for reading.
    FILE* file = fopen(path, "r");
    if (!file) {
        fprintf(stderr, "error: failed to open file '%s': '%s'\n", path, strerror(errno));
        return (FileContents){};
    }

    // To use the `stat` API, we need the file's descriptor.
    int file_descriptor = fileno(file);
    if (!file_descriptor) {
        fprintf(stderr, "error: failed to open file '%s': '%s'\n", path, strerror(errno));
        return (FileContents){};
    }

    struct stat stat_result;
    if (fstat(file_descriptor, &stat_result) != 0) {
        fprintf(stderr, "error: failed to read file '%s': '%s'\n", path, strerror(errno));
        return (FileContents){};
    }

    // Ensure that the filename passed is not actually a directory.
    if (S_ISDIR(stat_result.st_mode)) {
        fprintf(stderr, "error: '%s' is a directory, expected a file\n", path);
        return (FileContents){};
    }

    // If there is no file contents to read, return an empty struct.
    if (stat_result.st_size == 0) {
        return (FileContents){};
    }

    // Now that we know the file's size, we can allocate a buffer for its contents.
    char* contents = malloc(stat_result.st_size);
    if (!contents) {
#ifdef __APPLE__
        fprintf(stderr, "error: failed to allocate buffer of %lld bytes for '%s'\n", stat_result.st_size, path);
#else
        fprintf(stderr, "error: failed to allocate buffer of %zu bytes for '%s'\n", stat_result.st_size, path);
#endif
        return (FileContents){};
    }

    // We can now read the file's contents into that buffer.
    if (fread(contents, stat_result.st_size, sizeof(char), file) != 1) {
        fprintf(stderr, "error: failed to read file '%s': '%s'\n", path, strerror(errno));
        return (FileContents){};
    }

    fclose(file);
    return (FileContents){
        .data = contents,
        .length = stat_result.st_size,
    };
}

StringVector file_contents_lines(FileContents contents) {
    StringVector vector = vector_create();
    if (!vector_initialize(vector, 1)) {
        return vector;
    }

    auto current_line = string_builder_create();
    if (string_builder_is_invalid(current_line)) {
        return vector;
    }

    for (size_t i = 0; i < contents.length; i++) {
        // If the current builder is invalid, we need to create a new one for the next character.
        if (string_builder_is_invalid(current_line)) {
            current_line = string_builder_create();
        }

        auto character = contents.data[i];
        if (character == '\n') {
            vector_append(&vector, string_builder_finish(&current_line));
        } else {
            string_builder_append(&current_line, character);
        }
    }

    return vector;
}

void file_contents_destroy(FileContents contents) {
    free(contents.data);
}
