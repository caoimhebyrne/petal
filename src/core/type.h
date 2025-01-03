#pragma once

// Represents the different kinds of types.
typedef enum {
    // An unresolved type, just holds a string for the type's name.
    TYPE_KIND_UNRESOLVED,
} TypeKind;

// Represents a standard type.
typedef struct {
    // The kind of type that this is.
    TypeKind kind;
} Type;

// An unresolved type, the AST produces this for the typechecker to resolve and validate.
typedef struct {
    union {
        Type header;
    };

    // The name of the type, for example: "i32".
    char* name;
} UnresolvedType;

// Creates a new unresolved type.
// Parameters:
// - name: The name of the type.
// Returns: A reference to an `UnresolvedType` if successful, otherwise nullptr.
UnresolvedType* type_create_unresolved(char* name);

// De-allocates a Type.
// Parameters:
// - type: The type to destroy.
void type_destroy(Type* type);
