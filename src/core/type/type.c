#include "type.h"
#include "core/type/unresolved.h"
#include "core/type/value.h"
#include "util/format.h"
#include <stdlib.h>
#include <string.h>

char* type_to_string(Type* type) {
    switch (type->kind) {
    case TYPE_KIND_UNRESOLVED:
        return format_string("UnresolvedType ('%s')", ((UnresolvedType*)type)->name);

    case TYPE_KIND_VALUE:
        return format_string("%s", value_type_kind_to_string(((ValueType*)type)->value_kind));
    }
}

void type_destroy(Type* type) {
    switch (type->kind) {
    case TYPE_KIND_UNRESOLVED:
        auto unresolved_type = (UnresolvedType*)type;
        free(unresolved_type->name);

        break;

    case TYPE_KIND_VALUE:
        break;
    }

    free(type);
}
