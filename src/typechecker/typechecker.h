#pragma once

#include "ast/node.h"
#include "core/diagnostic.h"
#include "typechecker/context.h"
#include "typechecker/declared_function.h"
#include "typechecker/declared_type.h"

typedef struct {
    // A reference to the vector of nodes to type check.
    NodeVector* nodes;

    // A reference to a vector of diagnostics.
    DiagnosticVector* diagnostics;

    // The current context of the typechecker.
    TypecheckerContext context;

    // The functions declared during this typechecking session.
    DeclaredFunctionVector declared_functions;

    // The types declared during this typechecking session.
    DeclaredTypeVector declared_types;
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

// Destroys a typechecker.
void typechecker_destroy(Typechecker* typechecker);
