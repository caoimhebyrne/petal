#include "diagnostics.h"
#include "stream.h"
#include <stdarg.h>

void diagnostic_stream_print(DiagnosticStream* stream, char* filename) {
    for (size_t i = 0; i < stream->length; i++) {
        Diagnostic diagnostic = stream->data[i];

        char* prefix = diagnostic.is_terminal ? "error" : "warn";
        printf("%s: %s(%zu:%zu): %s\n", prefix, filename, diagnostic.position.line + 1, diagnostic.position.column,
               diagnostic.message);
    }
}

void diagnostic_stream_destroy(DiagnosticStream* stream) { free(stream->data); }

CREATE_STREAM(DiagnosticStream, diagnostic_stream, Diagnostic);
