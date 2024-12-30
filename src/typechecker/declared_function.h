#ifndef __TYPECHECKER_DECLARED_FUNCTION_H__
#define __TYPECHECKER_DECLARED_FUNCTION_H__

#include "../ast/parameter.h"
#include "../ast/type.h"
#include "../stream.h"

typedef struct {
    // The name of the function.
    char* name;

    // The return type of the function.
    ResolvedType* return_type;

    // The parameters that the function expects.
    Parameters parameters;
} DeclaredFunction;

DECLARE_STREAM(DeclaredFunctions, declared_functions, DeclaredFunction);

// Attempts to find a DeclaredFunction by its name.
// If no declared function with the provided name exists, 0 is returned.
// Parameters:
// - name: The name of the function to retrieve.
DeclaredFunction* declared_functions_find_by_name(DeclaredFunctions functions, char* name);

#endif // __TYPECHECKER_DECLARED_FUNCTION_H__
