#ifndef __MODULE_H__
#define __MODULE_H__

#include "ast/node.h"
#include "stream.h"
#include "typechecker/declared_function.h"
#include <llvm-c/Types.h>

typedef struct Module Module;

DECLARE_STREAM(ModuleDependencies, module_dependencies, Module);

// Represents a Petal module.
// This is typically a single file compiled into LLVM bytecode.
struct Module {
    // The original file name of this module.
    char* file_name;

    // A reference to the LLVM module compiled from this module.
    LLVMModuleRef llvm_module;

    // A reference to the LLVM context used for code generation.
    LLVMContextRef llvm_context;

    // The node stream used to compile this module.
    NodeStream node_stream;

    // The dependencies required to finish compiling this module.
    ModuleDependencies dependencies;
};

// Creates a new Module.
// Parameters:
// - file_name: The file to read source code from.
Module module_create(char* file_name);

// Creates a Module as a dependency of another Module.
Module module_create_dependency(Module* parent, char* file_name);

// Compiles this module.
bool module_compile(Module* module);

#endif // __MODULE_H__
