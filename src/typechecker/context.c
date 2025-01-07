#include "typechecker/context.h"
#include "util/vector.h"

bool typechecker_context_initialize(TypecheckerContext* context, Type* expected_return_type) {
    // Re-initialize the context.
    *context = (TypecheckerContext){0};
    context->expected_return_type = expected_return_type;
    context->declared_variables = (DeclaredVariableVector){};

    return vector_initialize(context->declared_variables, 1);
}

void typechecker_context_destroy(TypecheckerContext* context) {
    // Only free the declared variables if its value is not a null pointer.
    if (context->declared_variables.capacity != 0 || context->declared_variables.items != nullptr) {
        free(context->declared_variables.items);
    }

    context->expected_return_type = nullptr;
}
