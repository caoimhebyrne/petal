#include "allocator.h"
#include "array.h"
#include "file.h"
#include "logger.h"
#include <stdlib.h>

int main(const int argc, const char **argv, const char **envp) {
    (void)envp;

    if (argc != 2) {
        log_error("invalid usage, expected exactly one argument (path)");
        return EXIT_FAILURE;
    }

    const char* file_path = argv[1];
    if (!file_path) {
        log_error("invalid usage, expected exactly one argument (path)");
        return EXIT_FAILURE;
    }

    Allocator allocator = {0};
    allocator_init(&allocator);

    StringBuffer string_buffer = {0};
    string_buffer_init(&string_buffer, &allocator);

    if (!file_read(file_path, &string_buffer)) {
        return EXIT_FAILURE;
    }

    log_info("string buffer contents:");
    printf("%.*s", (int)string_buffer.length, string_buffer.data);

    allocator_clean(&allocator);

    return EXIT_SUCCESS;
}
