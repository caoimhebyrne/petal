#include "diagnostic.h"
#include "array.h"

IMPLEMENT_ARRAY_TYPE(DiagnosticArray, diagnostic_array, Diagnostic)

const char* diagnostic_kind_string(const DiagnosticKind kind) {
    switch (kind) {
    case DIAGNOSTIC_KIND_ERROR:
        return "error";

    default:
        return "unknown";
    }
}

void diagnostic_init(Diagnostic* diagnostic, const DiagnosticKind kind, const Position position, const char* message) {
    diagnostic->kind = kind;
    diagnostic->position = position;
    diagnostic->message = message;
}

void diagnostic_init_fmt(
    Diagnostic* diagnostic,
    Allocator* allocator,
    const DiagnosticKind kind,
    const Position position,
    const char* format,
    ...
) {
    va_list args;
    va_start(args, format);

    StringBuffer message = {0};
    string_buffer_init_vfmt(&message, allocator, format, args);

    va_end(args);

    // C strings must be null-terminated.
    string_buffer_append(&message, '\0');
    diagnostic_init(diagnostic, kind, position, message.data);
}
