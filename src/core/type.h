#pragma once

// Represents the different kinds of types.
#include "core/position.h"
typedef enum {
    // An unresolved type, just holds a string for the type's name.
    TYPE_KIND_UNRESOLVED,
} TypeKind;

// Represents a standard type.
typedef struct {
    // The kind of type that this is.
    TypeKind kind;

    // The position that this type occured at within the source file.
    Position position;
} Type;

// An unresolved type, the AST produces this for the typechecker to resolve and validate.
typedef struct {
    union {
        Type header;
    };

    // The name of the type, for example: "i32".
    char* name;
} UnresolvedType;

// A type which has been resolved by the typechecker.
typedef struct {
    union {
        Type header;
    };

    // TODO: Add value type?
} ResolvedType;

// Creates a new unresolved type.
// Parameters:
// - position: The position that this file occurred at within the source file.
// - name: The name of the type.
// Returns: A reference to an `UnresolvedType` if successful, otherwise nullptr.
UnresolvedType* type_create_unresolved(Position position, char* name);

// Creates a new resolved type.
// Parameters:
// Returns: A referenced to the `ResolvedType` if successful, otherwise nullptr.
ResolvedType* type_create_resolved();

// Returns a heap-allocated string represenatation of a Type.
char* type_to_string(Type* type);

// De-allocates a Type.
// Parameters:
// - type: The type to destroy.
void type_destroy(Type* type);
