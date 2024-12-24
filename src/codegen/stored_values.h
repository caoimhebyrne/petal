#ifndef __STORED_VALUES_H__
#define __STORED_VALUES_H__

#include "../stream.h"
#include <llvm-c/Types.h>

typedef struct {
    // The name of this value.
    char* name;

    // The LLVM value associated with this value.
    LLVMValueRef value;
} StoredValue;

DECLARE_STREAM(StoredValues, stored_values, StoredValue);

// Creates a stored value with a name and value.
StoredValue stored_value_create(char* name, LLVMValueRef value);

// Attempts to find a StoredValue by its name.
// If no stored value with the provided name exists, 0 is returned.
// Parameters:
// - name: The name of the value to retrieve.
StoredValue* stored_values_find_by_name(StoredValues values, char* name);

#endif // __STORED_VALUES_H__
