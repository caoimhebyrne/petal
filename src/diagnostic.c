#include "diagnostic.h"
#include "array.h"

IMPLEMENT_ARRAY_TYPE(DiagnosticArray, diagnostic_array, Diagnostic)

void diagnostic_init(Diagnostic* diagnostic, const DiagnosticKind kind, const ModuleId module_id, const char* message) {
    diagnostic->kind = kind;
    diagnostic->module_id = module_id;
    diagnostic->message = message;
}
