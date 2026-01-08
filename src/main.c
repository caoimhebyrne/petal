#include "allocator.h"
#include "array.h"
#include "file.h"
#include "logger.h"
#include <stdlib.h>

int main(const int argc, const char **argv, const char **envp) {
    (void)argc;
    (void)argv;
    (void)envp;

    Allocator allocator = {0};
    allocator_init(&allocator);

    StringBuffer string_buffer = {0};
    string_buffer_init(&string_buffer, &allocator);

    if (!file_read("./src/main.c", &string_buffer)) {
        return EXIT_FAILURE;
    }

    log_info("string buffer contents:");
    printf("%.*s", (int)string_buffer.length, string_buffer.data);

    allocator_clean(&allocator);

    return EXIT_SUCCESS;
}
