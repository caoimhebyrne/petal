#pragma once

#include "ast/node.h"
#include "core/diagnostic.h"
#include "typechecker/context.h"

typedef struct {
    // A reference to the vector of nodes to type check.
    NodeVector* nodes;

    // A reference to a vector of diagnostics.
    DiagnosticVector* diagnostics;

    // The current context of the typechecker.
    TypecheckerContext context;
} Typechecker;

// Creates a new Typechecker.
// Parameters:
// - nodes: A reference to the vector of nodes to type check.
// - diagnostics: A reference to a vector of diagnostics.
Typechecker typechecker_create(NodeVector* nodes, DiagnosticVector* diagnostics);

// Resolves and verifies types used in the typechecker's nodes.
// Parameters:
// - typechecker: The typechecker to use.
// Returns whether the typechecking was successful.
bool typechecker_check(Typechecker* typechecker);
