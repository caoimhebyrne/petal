#pragma once

#include "array.h"

/**
 * Represents the different kinds of types that exist.
 */
typedef enum {
    // The type has not been resolved yet.
    TYPE_KIND_UNKNOWN,

    // A void type. This type means "nothing", and is usually used when a function does not return a value.
    TYPE_KIND_VOID,
} TypeKind;

/**
 * Represents a type in the petal compiler.
 */
typedef struct {
    /**
     * The kind of type that this is.
     */
    const TypeKind kind;

    union {
        /**
         * The name of a TYPE_KIND_UNKNOWN.
         */
        const StringBuffer type_name;
    };
} Type;

/**
 * Creates a new unknown type kind with the provided type name/
 */
Type type_unknown(const StringBuffer type_name);
