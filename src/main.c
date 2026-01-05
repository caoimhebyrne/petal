#include "allocator.h"
#include "array.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

int main(const int argc, const char** argv, const char** envp) {
    (void)argc;
    (void)argv;
    (void)envp;

    printf("Hello, world!\n");

    Allocator allocator = {0};
    allocator_init(&allocator);
    allocator_clean(&allocator);

    return EXIT_SUCCESS;
}
