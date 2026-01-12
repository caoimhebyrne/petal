#pragma once

#include "array.h"
#include "module_id.h"

/**
 * The different kinds of diagnostics that exist.
 */
typedef enum {
    // An internal compiler error, this will typically prevent the compilation from executing further.
    DIAGNOSTIC_KIND_INTERNAL_ERROR,
} DiagnosticKind;

// Returns a human readable string ('internal error', 'error', 'warning', etc.) for a diagnostic kind.
const char* diagnostic_kind_string(const DiagnosticKind kind);

/**
 * A diagnostic is typically an error or a warning that occurs while parsing or executing source code.
 */
typedef struct {
    // The kind of diagnostic that this is.
    DiagnosticKind kind;

    // The ID of the module that the diagnostic occurred in.
    ModuleId module_id;

    // The message associated with this diagnostic.
    const char* message;
} Diagnostic;

/**
 * Initializes a new [Diagnostic] with the provided kind, module ID, and message.
 */
void diagnostic_init(Diagnostic* diagnostic, const DiagnosticKind kind, const ModuleId module_id, const char* message);

DEFINE_ARRAY_TYPE(DiagnosticArray, diagnostic_array, Diagnostic)
