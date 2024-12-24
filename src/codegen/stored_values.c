#include "stored_values.h"
#include <string.h>

CREATE_STREAM(StoredValues, stored_values, StoredValue);

StoredValue stored_value_create(char* name, LLVMValueRef value) { return (StoredValue){name, value}; }

StoredValue* stored_values_find_by_name(StoredValues values, char* name) {
    for (size_t i = 0; i < values.length; i++) {
        StoredValue* value = &values.data[i];
        if (strcmp(value->name, name) == 0) {
            return value;
        }
    }

    return 0;
}

void stored_values_destroy(StoredValues* stream) {
    for (size_t i = 0; i < stream->length; i++) {
        StoredValue value = stream->data[i];
        free(value.name);
    }

    free(stream->data);
}
