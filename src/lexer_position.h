#pragma once

// Prevents a circular dependency between lexer.h and diagnostic.h

#include "module_id.h"

/**
 * A position that a token occurred at within a source file.
 */
typedef struct {
    // The module that the token was in.
    ModuleId module_id;

    // The line that the token started at.
    size_t line;

    // The column that the token started at.
    size_t column;

    // The length of the token.
    size_t length;
} Position;
