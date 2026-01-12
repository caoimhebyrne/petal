#pragma once

#include <stddef.h>

// This belongs here to prevent a circular dependency between diagnostic.h and module.h
typedef struct {
    size_t unwrap;
} ModuleId;
