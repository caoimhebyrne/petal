#pragma once

#include "core/type/type.h"

// The current context of the typechecker.
typedef struct {
    // The return type of the current block (usually a function declaration).
    Type* expected_return_type;
} TypecheckerContext;
