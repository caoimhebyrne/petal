#pragma once

#define ANSI_COLOR_GRAY "\x1b[90m"
#define ANSI_COLOR_RESET "\x1b[0m"
#define ANSI_COLOR_GREEN "\x1b[32m"
#define ANSI_COLOR_YELLOW "\x1b[33m"

// Whether debug logging should be enabled.
extern bool enable_debug_logging;

#define LOG_SUCCESS(msg, ...) printf(ANSI_COLOR_GREEN "success" ANSI_COLOR_RESET ": " msg "\n", ##__VA_ARGS__)
#define LOG_WARNING(msg, ...) printf(ANSI_COLOR_YELLOW "warning" ANSI_COLOR_RESET ": " msg "\n", ##__VA_ARGS__)

#define LOG_DEBUG(group, msg, ...)                                                                                     \
    ({                                                                                                                 \
        if (enable_debug_logging) {                                                                                    \
            printf(                                                                                                    \
                ANSI_COLOR_GRAY "debug"                                                                                \
                                "(%s): " msg ANSI_COLOR_RESET "\n",                                                    \
                group,                                                                                                 \
                ##__VA_ARGS__                                                                                          \
            );                                                                                                         \
        }                                                                                                              \
    })
