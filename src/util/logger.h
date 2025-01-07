#pragma once

#define ANSI_COLOR_GRAY "\x1b[90m"
#define ANSI_COLOR_RESET "\x1b[0m"

// Whether debug logging should be enabled.
static bool enable_debug_logging = true;

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
