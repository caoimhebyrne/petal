#pragma once

#include "core/type/type.h"
#include "typechecker/declared_variable.h"

// The current context of the typechecker.
typedef struct {
    // The return type of the current block (usually a function declaration).
    Type* expected_return_type;

    // The declared variables within this context.
    DeclaredVariableVector declared_variables;
} TypecheckerContext;

// Initializes a TypecheckerContext.
// Parameters:
// - expected_return_type: The return type of the current block (usually a function declaration).
bool typechecker_context_initialize(TypecheckerContext* context, Type* expected_return_type);

// Destroys a TypecheckerContext, setting `expected_return_type` to `nullptr` and free'ing `declared_variables`.
// Parameters:
// - context: The context to destroy.
void typechecker_context_destroy(TypecheckerContext* context);
