#pragma once

#include "type.h"

// An unresolved type, the AST produces this for the typechecker to resolve and validate.
typedef struct {
    union {
        Type header;
    };

    // The name of the type, for example: "i32".
    char* name;
} UnresolvedType;

// Creates a new UnresolvedType.
// Parameters:
// - position: The position that this file occurred at within the source file.
// - name: The name of the type.
// Returns: A reference to an `UnresolvedType` if successful, otherwise nullptr.
UnresolvedType* unresolved_type_create(Position position, char* name);
