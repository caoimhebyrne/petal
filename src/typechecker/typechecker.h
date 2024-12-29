#ifndef __TYPECHECKER_H__
#define __TYPECHECKER_H__

#include "../ast/node.h"
#include "../diagnostics.h"
#include "declared_variable.h"

typedef struct {
    // The diagnostics produced from this typechecker.
    DiagnosticStream diagnostics;

    // The variables found during this typechecking session.
    DeclaredVariables variables;
} Typechecker;

// Creates a new TypeChecker.
Typechecker typechecker_create();

// Attempts to infer and verify types on the provided NodeStream.
// This operation is only successful if `typechecker.diagnostics.length` is equal to zero.
// Parameters:
// - node_stream: The NodeStream to infer & verify types on.
void typechecker_run(Typechecker* typechecker, NodeStream* node_stream);

// Destroys a TypeChecker.
// Parameters:
// - typechecker: The TypeChecker to destroy.
void typechecker_destroy(Typechecker* typechecker);

#endif // __TYPECHECKER_H__
