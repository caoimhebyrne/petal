#ifndef __TYPECHECKER_TYPE_ALIAS_H__
#define __TYPECHECKER_TYPE_ALIAS_H__

#include "../ast/type.h"
#include "../stream.h"

typedef struct {
    // The name of the type alias.
    char* name;

    // The actual type being aliased.
    ResolvedType* type;
} TypeAlias;

DECLARE_STREAM(TypeAliases, type_aliases, TypeAlias);

// Attempts to find a TypeAlias by its name.
// If no type alias with the provided name exists, 0 is returned.
// Parameters:
// - name: The name of the type alias to retrieve.
TypeAlias* type_aliases_find_by_name(TypeAliases aliases, char* name);

#endif // __TYPECHECKER_TYPE_ALIAS_H__
