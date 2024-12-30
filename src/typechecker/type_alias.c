#include "type_alias.h"

CREATE_STREAM(TypeAliases, type_aliases, TypeAlias);

void type_aliases_destroy(TypeAliases* aliases) { free(aliases->data); }

TypeAlias* type_aliases_find_by_name(TypeAliases aliases, char* name) {
    for (size_t i = 0; i < aliases.length; i++) {
        TypeAlias* alias = &aliases.data[i];
        if (strcmp(alias->name, name) == 0) {
            return alias;
        }
    }

    return 0;
}
