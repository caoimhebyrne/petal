#ifndef __LOGGER_H__
#define __LOGGER_H__

#define ANSI_COLOR_RED "\x1b[31m"
#define ANSI_COLOR_GREEN "\x1b[32m"
#define ANSI_COLOR_YELLOW "\x1b[33m"
#define ANSI_COLOR_BLUE "\x1b[34m"
#define ANSI_COLOR_MAGENTA "\x1b[35m"
#define ANSI_COLOR_CYAN "\x1b[36m"
#define ANSI_COLOR_LIGHT_GRAY "\x1b[90m"
#define ANSI_COLOR_GRAY "\x1b[90m"
#define ANSI_COLOR_RESET "\x1b[0m"

#include <stdio.h>

#define LOG_SUCCESS(msg, ...) printf(ANSI_COLOR_GREEN "success" ANSI_COLOR_RESET ": " msg "\n", ##__VA_ARGS__)

#define LOG_INFO(group, msg, ...)                                                                                      \
    printf(ANSI_COLOR_BLUE "info" ANSI_COLOR_RESET "(%s): " msg "\n", group, ##__VA_ARGS__)

#ifdef DEBUG
#define LOG_DEBUG(group, msg, ...)                                                                                     \
    printf(ANSI_COLOR_GRAY "debug"                                                                                     \
                           "(%s): " msg ANSI_COLOR_RESET "\n",                                                         \
           group, ##__VA_ARGS__)
#else
#define LOG_DEBUG(group, msg, ...)
#endif

#define LOG_ERROR(group, msg, ...)                                                                                     \
    fprintf(stderr, ANSI_COLOR_RED "error" ANSI_COLOR_RESET "(%s): " msg "\n", group, ##__VA_ARGS__)

#define LOG_WARNING(group, msg, ...)                                                                                   \
    fprintf(stderr, ANSI_COLOR_YELLOW "warning" ANSI_COLOR_RESET "(%s): " msg "\n", group, ##__VA_ARGS__)

#define LOG_TODO(group, msg, ...) fprintf(stderr, "todo(%s): " msg "\n", group, ##__VA_ARGS__)

#endif
