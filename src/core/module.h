#pragma once

// A "module" is any file that is being compiled, it may be a dependency resolved by another module.
#include "core/diagnostic.h"

typedef struct {
    // A vector of diagnostics produced for this module.
    DiagnosticVector diagnostics;

    // The original filename for this module.
    char* file_name;
} Module;

// Creates a new Module.
// Parameters:
// - file_name: The name of the Petal file to compile.
//              This `Module` will take ownership of this file name, it must not be used after calling `module_destroy`.
Module module_create(char* file_name);

// Initializes a module.
// Parameters:
// - module: The module to initialize.
// Returns whether the initialization was a success.
bool module_initialize(Module* module);

// Fully compiles this Module from lexing to code-generation.
// Parameters:
// - module: The module to compile.
void module_compile(Module* module);

// Destroys a Module.
// This will call `module_destroy` on all of its dependencies.
// Parameters:
// - module: The module to destroy.
void module_destroy(Module module);
