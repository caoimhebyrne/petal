#include "vm_builtin.h"
#include "logger.h"

VMValue petal_vm_builtin_print(const PetalBuiltinContext* context) {
    const VMValue* value = petal_builtin_arguments_get(context->arguments, 0);
    if (value->kind != VM_VALUE_KIND_STRING) {
        log_error("vm: expected a string to be passed to 'print' builtin!");
        return (VMValue){.kind = VM_VALUE_NOTHING};
    }

    printf("%.*s\n", (int)value->string.length, value->string.data);
    return (VMValue){.kind = VM_VALUE_NOTHING};
}

VMBuiltinFunction petal_vm_builtin_functions[] = {
    (VMBuiltinFunction){.name = "print", .handler = petal_vm_builtin_print},
};

const size_t petal_vm_builtin_functions_count = sizeof(petal_vm_builtin_functions) / sizeof(VMBuiltinFunction);
