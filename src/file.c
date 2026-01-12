#include "file.h"
#include "array.h"
#include "logger.h"
#include <errno.h>
#include <stdio.h>
#include <string.h>
#include <sys/stat.h>

bool file_read(const char* path, StringBuffer* output) {
    FILE* file = fopen(path, "r");
    if (!file) {
        log_error("could not read file '%s': %s", path, strerror(errno));
        goto fail;
    }

    const int file_descriptor = fileno(file);
    if (!file_descriptor) {
        log_error("could not get descriptor for file '%s': %s", path, strerror(errno));
        goto fail;
    }

    struct stat file_stat;
    if (fstat(file_descriptor, &file_stat) != 0) {
        log_error("could not stat file '%s': %s", path, strerror(errno));
        goto fail;
    }

    // If this is not a standard file, then we should not do anything with it.
    if (!S_ISREG(file_stat.st_mode)) {
        log_error("could not open file '%s': not a regular file");
        goto fail;
    }

    // If the file has no size, then we do not need to read anything.
    if (file_stat.st_size == 0) {
        goto success;
    }

    // We must ensure that the string buffer has enough memory allocated for the file's size.
    string_buffer_resize(output, sizeof(char) * file_stat.st_size);

    // We can then read the file into the buffer.
    if (fread(output->data, output->capacity, sizeof(char), file) != 1) {
        log_error("could not read file '%s': %s", path, strerror(errno));
        goto fail;
    }

    // We have written our data to the buffer, we must make sure that its length shows that.
    output->length = output->capacity;

success:
    if (file)
        fclose(file);
    return true;

fail:
    if (file)
        fclose(file);
    return false;
}
