#include "allocator.h"
#include "array.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

int main(const int argc, const char** argv, const char** envp) {
    (void)argc;
    (void)argv;
    (void)envp;

    Allocator allocator = {0};
    allocator_init(&allocator);

    StringBuffer string_buffer = {0};
    string_buffer_init(&string_buffer, &allocator);
    string_buffer_append_many(&string_buffer, "Hello, world!", 12);

    printf("%*s\n", (int)string_buffer.length, string_buffer.data);

    allocator_clean(&allocator);

    return EXIT_SUCCESS;
}
