#ifndef __POSITION_H__
#define __POSITION_H__

#include <stddef.h>

typedef struct Position {
    // The line that this position is referring to.
    size_t line;

    // The column that this position is referring to.
    size_t column;

    // The raw index into the data that this position is referring to.
    size_t index;
} Position;

void position_advance(Position* position);
void position_retreat(Position* position);
void position_advance_line(Position* position);

#endif // __POSITION_H__
