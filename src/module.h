#pragma once

#include "allocator.h"
#include "array.h"
#include "ast_statement.h"
#include "diagnostic.h"
#include "module_id.h"
#include <stdbool.h>

typedef struct {
    // The unique identifier for this module.
    ModuleId id;

    // The allocator used by this module.
    Allocator* allocator;

    // The [DiagnosticArray] to emit diagnostics on to.
    DiagnosticArray* diagnostics;

    // The path that this module lives at (relative to the current working directory).
    StringBuffer file_path;

    // The name of this module.
    StringBuffer name;

    // The source code of this module.
    StringBuffer source;
} Module;

// Initializes a module by reading the source code from a [file_path].
bool module_init(Module* module, Allocator* allocator, DiagnosticArray* diagnostics, const char* file_path);

// Parses the provided module's source code.
bool module_parse(Module* module, StatementArray* statements);
