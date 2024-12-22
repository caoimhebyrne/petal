#ifndef __DIAGNOSTICS_H__
#define __DIAGNOSTICS_H__

#include "position.h"
#include "stream.h"

// Represents a single diagnostic that can be produced.
typedef struct {
    // The position that this error occurred at.
    Position position;

    // The message associated with this diagnostic.
    char* message;

    // Whether this diagnostic is terminal or not (i.e. unrecoverable).
    bool is_terminal;
} Diagnostic;

DECLARE_STREAM(DiagnosticStream, diagnostic_stream, Diagnostic);

// Resets the provided DiagnoticStream with a certain capacity.
// Parameters:
// - diagnostic_stream: The diagnostic stream to reset.
// - initial_capacity: The initial capacity of the new stream.
bool diagnostic_stream_reset(DiagnosticStream* stream, size_t initial_capacity);

// Prints the diagnostic stream out to the console.
// Parameters:
// - diagnostic_stream: The diagnostic stream to print.
// - filename: The name of the file that the diagnostics occurred in.
void diagnostic_stream_print(DiagnosticStream* stream, char* filename);

#endif // __DIAGNOSTICS_H__
