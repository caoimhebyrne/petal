#ifndef __TYPE_H__
#define __TYPE_H__

#include "type-kind.h"
#include <stdbool.h>

// There are two kinds of types, resolved types and unresolved types.
// - An unresolved type is one that the AST parser recognizes as a type, but it is unsure
//   of its validity.
// - A resolved type has been resolved and verified by the typechecker, and is safe
//   to use during code-generation.
typedef struct {
    // Whether this type has been resolved or not.
    // If true, can be casted to `ResolvedType`.
    // If false, can be casted to `UnresolvedType`.
    bool is_resolved;

    // Whether this type is optional or not.
    bool is_optional;

    // Whether this type is a reference or not.
    bool is_reference;
} Type;

// An unresolved type.
// This has been parsed by the AST parser, but not verified yet (i.e. it may not exist).
typedef struct {
    // Whether this type has been resolved or not, always false.
    bool is_resolved;

    // Whether this type is optional or not.
    bool is_optional;

    // Whether this type is a reference or not.
    bool is_reference;

    // The name of the type being referenced.
    char* name;
} UnresolvedType;

// Creates a new unresolved type.
// Parameters:
// - is_reference: Whether the type being referred to is a reference.
// - name: The name being used to refer to this type.
// Returns:
// - An unresolved type reference if successful, otherwise 0.
UnresolvedType* type_create_unresolved(bool is_optional, bool is_reference, char* name);

// A resolved type.
// This has been resolved and verified as valid by the typechecker, and is safe to use
// for code-generation.
typedef struct {
    // Whether this type has been resolved or not, always true.
    bool is_resolved;

    // Whether this type is optional or not.
    bool is_optional;

    // Whether this type is a reference or not.
    bool is_reference;

    // The kind of type that this has been resolved to.
    TypeKind kind;
} ResolvedType;

// Creates a new resolved type.
// Parameters:
// - is_reference: Whether the type being referred to is a reference.
// - kind: The kind of type that has been resolved.
// Returns:
// - A resolved type reference if successful, otherwise 0.
ResolvedType* type_create_resolved(bool is_optional, bool is_reference, TypeKind kind);

// Checks if two types are equal.
// If both types are resolved, the kind must be the same.
// If both types are unresolved, the name must be the same.
// Parameters:
// - type_a: The first type.
// - type_b: The second type.
bool type_equal(Type* type_a, Type* type_b);

// Returns a string representation of the provided type.
// Parameters:
// - type: The type to stringify.
char* type_to_string(Type* type);

// De-allocates a type.
// Parameters:
// - type: The type to de-allocate.
void type_destroy(Type* type);

#endif // __TYPE_H__
