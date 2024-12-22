#ifndef __FORMAT_STRING_H__
#define __FORMAT_STRING_H__

#include <stdarg.h>
#include <stdio.h>
#include <stdlib.h>

static inline char* format_string(const char* format, ...) {
    va_list args;
    va_start(args, format);
    int len = vsnprintf(0, 0, format, args);
    va_end(args);

    if (len < 0) {
        return 0;
    }

    char* buffer = (char*)malloc(len + 1);
    if (!buffer) {
        return 0;
    }

    va_start(args, format);
    vsnprintf(buffer, len + 1, format, args);
    va_end(args);

    return buffer;
}

#endif // __FORMAT_STRING_H__
