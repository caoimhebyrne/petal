#ifndef __CORE_MODULE_H__
#define __CORE_MODULE_H__

// A "module" is any file that is being compiled, it may be a dependency resolved by another module.
typedef struct {
    // The original filename for this module.
    char* file_name;
} Module;

// Initializes a new Module.
// Parameters:
// - file_name: The name of the Petal file to compile.
//              This `Module` will take ownership of this file name, it must not be used after calling `module_destroy`.
Module module_create(char* file_name);

// Fully compiles this Module from lexing to code-generation.
// Parameters:
// - module: The module to compile.
void module_compile(Module* module);

// Destroys a Module.
// This will call `module_destroy` on all of its dependencies.
// Parameters:
// - module: The module to destroy.
void module_destroy(Module module);

#endif // __CORE_MODULE_H__
