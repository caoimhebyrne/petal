#ifndef __CORE_POSITION_H__
#define __CORE_POSITION_H__

#include <stddef.h>

// Represents a position within a source file that is being compiled
typedef struct {
    // The raw character index that this node started at.
    size_t index;

    // The line number that this node started on (0-based index).
    size_t line;

    // The column that this node started on (0-based index).
    size_t column;

    // The length of this node in characters.
    size_t length;
} Position;

#endif // __CORE_POSITION_H__
