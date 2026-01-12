#pragma once

#include <stdarg.h>
#include <stdio.h>

#define COLOR_RED "\033[31m"
#define COLOR_GREY "\033[30m"
#define COLOR_WHITE "\033[37m"
#define COLOR_GREEN "\033[32m"
#define COLOR_RESET "\033[0m"

static inline void log_debug(const char* format, ...) {
    va_list arguments;
    va_start(arguments, format);

    printf(COLOR_GREY "debug: ");
    vprintf(format, arguments);
    printf(COLOR_RESET "\n");

    va_end(arguments);
}

static inline void log_info(const char* format, ...) {
    va_list arguments;
    va_start(arguments, format);

    printf(COLOR_GREEN "info: " COLOR_RESET);
    vprintf(format, arguments);
    printf(COLOR_RESET "\n");

    va_end(arguments);
}

static inline void log_error(const char* format, ...) {
    va_list arguments;
    va_start(arguments, format);

    fprintf(stderr, COLOR_RED "error: " COLOR_RESET);
    vfprintf(stderr, format, arguments);
    fprintf(stderr, COLOR_RESET "\n");

    va_end(arguments);
}
