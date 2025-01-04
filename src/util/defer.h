#pragma once

#include <stdlib.h>

#define defer(fn) __attribute__((__cleanup__(fn)))

[[maybe_unused]] static void free_str(void* ptr) {
    if (ptr) {
        free(*(void**)ptr);
    }
}
