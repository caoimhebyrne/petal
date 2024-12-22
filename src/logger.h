#ifndef __LOGGER_H__
#define __LOGGER_H__

#include <stdio.h>

#define LOG_INFO(group, msg, ...) printf("info(%s): " msg "\n", group, ##__VA_ARGS__)

#ifdef DEBUG
#define LOG_DEBUG(group, msg, ...) printf("debug(%s): " msg "\n", group, ##__VA_ARGS__)
#else
#define LOG_DEBUG(group, msg, ...)
#endif

#define LOG_ERROR(group, msg, ...) fprintf(stderr, "error(%s): " msg "\n", group, ##__VA_ARGS__)
#define LOG_TODO(group, msg, ...) fprintf(stderr, "todo(%s): " msg "\n", group, ##__VA_ARGS__)

#endif
