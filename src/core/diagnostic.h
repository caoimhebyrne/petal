#pragma once

#include "core/position.h"
#include "util/vector.h"

#define ANSI_RED "\e[0;31m"
#define ANSI_YELLOW "\e[0;33m"
#define ANSI_RESET "\e[0m"
#define ANSI_GRAY "\e[1;30m"
#define ANSI_LIGHT_GRAY "\e[1;37m"

typedef struct {
    // The position in the source file that this diagnostic occurred at.
    Position position;

    // The heap-allocated message message to display with this diagnostic.
    char* message;
} Diagnostic;

#define DIAGNOSTIC_INTERNAL_ERROR(pos)                                                                                 \
    (Diagnostic) {                                                                                                     \
        .position = pos, .message = format_string("internal compiler error"),                                          \
    }

// Creates a new Diagnostic.
// Parameters:
// - position: The position in the source file that this diagnostic occurred at.
// - message: The heap-allocated message message to display with this diagnostic.
Diagnostic diagnostic_create(Position position, char* message);

// De-allocates a Diagnostic's data
// Parameters:
// - diagnostic: The diagnostic to destroy.
void diagnostic_destroy(Diagnostic diagnostic);

// A vector of `Diagnostic`s.
typedef Vector(Diagnostic) DiagnosticVector;
