#include "file.h"

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

void file_contents_destroy(FileContents contents) {
    free(contents.data);
}
