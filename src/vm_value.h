#pragma once

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
    };
} VMValue;
