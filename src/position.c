#include "position.h"

void position_advance(Position* position) {
    position->column += 1;
    position->index += 1;
}

void position_retreat(Position* position) {
    position->column -= 1;
    position->index -= 1;
}

void position_advance_line(Position* position) {
    position->column = 0;
    position->line += 1;
}
