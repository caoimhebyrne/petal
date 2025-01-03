#pragma once

#include <stdarg.h>
#include <stdio.h>
#include <stdlib.h>

static inline char* format_string(const char* format, ...) {
    va_list args;
    va_start(args, format);

    auto length = vsnprintf(0, 0, format, args);
    va_end(args);

    if (length < 0) {
        return nullptr;
    }

    char* buffer = (char*)malloc(length + 1);
    if (!buffer) {
        return nullptr;
    }

    va_start(args, format);
    vsnprintf(buffer, length + 1, format, args);
    va_end(args);

    return buffer;
}
