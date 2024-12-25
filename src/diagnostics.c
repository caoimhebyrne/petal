#include "diagnostics.h"
#include "logger.h"
#include "stream.h"
#include <stdarg.h>

void diagnostic_stream_print(DiagnosticStream* stream, char* filename) {
    for (size_t i = 0; i < stream->length; i++) {
        Diagnostic diagnostic = stream->data[i];

        if (diagnostic.is_terminal) {
            printf(ANSI_COLOR_RED "error" ANSI_COLOR_RESET ": %s(%zu:%zu): %s\n", filename,
                   diagnostic.position.line + 1, diagnostic.position.column, diagnostic.message);
        } else {
            printf(ANSI_COLOR_YELLOW "warning" ANSI_COLOR_RESET ": %s(%zu:%zu): %s\n", filename,
                   diagnostic.position.line + 1, diagnostic.position.column, diagnostic.message);
        }
    }
}

void diagnostic_stream_destroy(DiagnosticStream* stream) { free(stream->data); }

CREATE_STREAM(DiagnosticStream, diagnostic_stream, Diagnostic);
