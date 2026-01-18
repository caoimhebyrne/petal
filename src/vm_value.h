#pragma once

#include "array.h"

/**
 * Represents the different kinds of values that can be used within a virtual machine.
 */
typedef enum {
    /**
     * Nothing.
     */
    VM_VALUE_NOTHING,

    /**
     * A number (C float).
     */
    VM_VALUE_KIND_NUMBER,

    /**
     * A string.
     */
    VM_VALUE_KIND_STRING,
} VMValueKind;

/**
 * Represents a value used by the virtual machine.
 */
typedef struct {
    /**
     * The kind of value that this is.
     */
    VMValueKind kind;

    union {
        /**
         * VM_VALUE_KIND_NUMBER.
         */
        float number;

        /**
         * VM_VALUE_KIND_STRING.
         */
        StringBuffer string;
    };
} VMValue;

DEFINE_ARRAY_TYPE(VMValueArray, vm_value_array, VMValue)
