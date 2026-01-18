#include "ast_type.h"
#include "array.h"

Type type_unknown(const StringBuffer type_name) {
    return (Type){.kind = TYPE_KIND_UNKNOWN, .type_name = type_name};
}
