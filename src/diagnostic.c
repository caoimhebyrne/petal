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
