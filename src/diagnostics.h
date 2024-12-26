#ifndef __DIAGNOSTICS_H__
#define __DIAGNOSTICS_H__

#include "position.h"
#include "stream.h"
#include "string/format_string.h"

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

// Creates and appends a Diagnostic to the diagnostic stream.
// Parameters:
// - diagnostic_stream: The diagnostic stream to append to.
// - pos: The position that the diagnostic occurred at.
// - terminal: Whether this diagnostic is terminal or not (i.e. unrecoverable).
// - format: The message associated with this diagnostic.
#define diagnostic_stream_push(stream, pos, terminal, format, ...)                                                     \
    {                                                                                                                  \
        Diagnostic diagnostic = {                                                                                      \
            .position = pos,                                                                                           \
            .message = format_string(format, ##__VA_ARGS__),                                                           \
            .is_terminal = terminal,                                                                                   \
        };                                                                                                             \
        if (!diagnostic_stream_append(stream, diagnostic)) {                                                           \
            LOG_ERROR("diagnostic", "failed to append diagnostic");                                                    \
        }                                                                                                              \
    }

// Prints the diagnostic stream out to the console.
// Parameters:
// - diagnostic_stream: The diagnostic stream to print.
// - filename: The name of the file that the diagnostics occurred in.
void diagnostic_stream_print(DiagnosticStream* stream, char* filename);

#endif // __DIAGNOSTICS_H__
