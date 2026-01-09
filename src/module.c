#include "module.h"
#include "array.h"
#include "file.h"
#include <string.h>

bool module_init(Module* module, Allocator* allocator, const char* file_path) {
    StringBuffer file_path_buffer = {0};
    string_buffer_init(&file_path_buffer, allocator);
    string_buffer_append_many(&file_path_buffer, file_path, strlen(file_path));

    StringBuffer source_buffer = {0};
    string_buffer_init(&source_buffer, allocator);

    if (!file_read(file_path, &source_buffer)) {
        return false;
    }

    module->allocator = allocator;
    module->file_path = file_path_buffer;
    module->source = source_buffer;

    return true;
}
