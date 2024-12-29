#ifndef __TYPECHECKER_DECLARED_VARIABLE_H__
#define __TYPECHECKER_DECLARED_VARIABLE_H__

#include "../ast/type.h"
#include "../stream.h"

typedef struct {
    // The name of the variable.
    char* name;

    // The type that this variable expects.
    Type type;
} DeclaredVariable;

DECLARE_STREAM(DeclaredVariables, declared_variables, DeclaredVariable);

// Attempts to find a DeclaredVariables by its name.
// If no declared variable with the provided name exists, 0 is returned.
// Parameters:
// - name: The name of the variable to retrieve.
DeclaredVariable* declared_variables_find_by_name(DeclaredVariables variables, char* name);

#endif // __TYPECHECKER_DECLARED_VARIABLE_H__
