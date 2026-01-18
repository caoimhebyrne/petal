#include "vm.h"
#include "allocator.h"
#include "array.h"
#include "ast_statement.h"
#include "logger.h"
#include "vm_value.h"
#include <assert.h>

VMValue petal_vm_builtin_print(PetalVM* vm, const VMValueArray* arguments) {
    (void)vm;

    // TODO: This is really bad...
    if (arguments->length != 1) {
        return (VMValue){.kind = VM_VALUE_NOTHING};
    }

    VMValue value = arguments->data[0];
    if (value.kind != VM_VALUE_KIND_STRING) {
        return (VMValue){.kind = VM_VALUE_NOTHING};
    }

    printf("%.*s\n", (int)value.string.length, value.string.data);

    return (VMValue){.kind = VM_VALUE_NOTHING};
}

VMBuiltinFunction builtin_functions[] = {(VMBuiltinFunction){.name = "print", .handler = petal_vm_builtin_print}};

IMPLEMENT_ARRAY_TYPE(FunctionDeclarationArray, function_declaration_array, FunctionDeclarationStatement)
IMPLEMENT_ARRAY_TYPE(VMVariableArray, vm_variable_array, VMVariable)
IMPLEMENT_ARRAY_TYPE(VMValueArray, vm_value_array, VMValue)

/**
 * Attempts to find a function in the VM's current state with the provided name, returning NULL if it could not be
 * found.
 */
const FunctionDeclarationStatement* petal_vm_get_function(PetalVM* vm, const StringBuffer* function_name);

/**
 * Attempts to execute the provided statement, updating the VM state if neccessary.
 */
bool petal_vm_exec_statement(PetalVM* vm, VMScope* scope, Statement* statement);

/**
 * Evaluates the provided expression. If a nothing value is returned, the evaluation failed.
 */
VMValue petal_vm_eval_expression(PetalVM* vm, VMScope* scope, const Expression* expression);

/**
 * Attempts to call the function with the provided arguments.
 */
bool petal_vm_call_function(
    PetalVM* vm,
    VMScope* scope,
    const FunctionDeclarationStatement* function,
    const ExpressionArray* arguments
);

/**
 * Attempts to find a built-in function with the provided name.
 */
const VMBuiltinFunction* petal_vm_get_builtin_function(PetalVM* vm, const FunctionCall* call);

/**
 * Attempts to get and call the function using the information in the [FunctionCall].
 */
bool petal_vm_get_and_call_function(PetalVM* vm, VMScope* scope, const FunctionCall* call);

void vm_scope_init(VMScope* scope, Allocator* allocator) {
    assert(scope != NULL && "NULL VMScope passed to vm_scope_init");
    assert(allocator != NULL && "NULL Allocator passed to vm_scope_init");

    scope->return_value = (VMValue){0};
    scope->stop_execution = false;

    vm_variable_array_init(&scope->variables, allocator);
}

void petal_vm_init(PetalVM* vm, Allocator* allocator, const StatementArray* statements) {
    assert(vm != NULL && "NULL PetalVM passed to petal_vm_init");
    assert(allocator != NULL && "NULL Allocator passed to petal_vm_init");
    assert(statements != NULL && "NULL StatementArray passed to petal_vm_init");

    vm->allocator = allocator;
    vm->state = (VMState){0};
    vm->functions = (FunctionDeclarationArray){0};

    function_declaration_array_init(&vm->functions, allocator);

    for (size_t i = 0; i < statements->length; i++) {
        const Statement* statement = statements->data[i];
        if (statement->kind != STATEMENT_KIND_FUNCTION_DECLARATION) {
            continue;
        }

        function_declaration_array_append(&vm->functions, statement->function_declaration);
    }
}

bool petal_vm_exec(PetalVM* vm) {
    assert(vm != NULL && "NULL PetalVM passed to petal_vm_exec");

    FunctionCall main_function_call = {0};
    string_buffer_init_from_cstr(&main_function_call.name, vm->allocator, "main");

    VMScope scope = {0};
    vm_scope_init(&scope, vm->allocator);

    if (!petal_vm_get_and_call_function(vm, &scope, &main_function_call)) {
        return false;
    }

    if (scope.return_value.kind != VM_VALUE_KIND_NUMBER) {
        log_error("vm: main function did not return an integer!");
        return false;
    }

    vm->state.exit_code = (int)scope.return_value.number;
    return true;
}

bool petal_vm_exec_statement(PetalVM* vm, VMScope* scope, Statement* statement) {
    (void)vm;

    switch (statement->kind) {
    case STATEMENT_KIND_RETURN: {
        scope->stop_execution = true;

        if (statement->return_.value) {
            scope->return_value = petal_vm_eval_expression(vm, scope, statement->return_.value);
        }

        break;
    }

    case STATEMENT_KIND_FUNCTION_CALL: {
        VMScope call_scope = {0};
        vm_scope_init(&call_scope, vm->allocator);

        if (!petal_vm_get_and_call_function(vm, &call_scope, &statement->function_call)) {
            return false;
        }

        break;
    }

    case STATEMENT_KIND_FUNCTION_DECLARATION:
        log_error("vm: encountered a top-level statement kind while executing scope-based statement?");
        return false;
    }

    return true;
}

VMValue petal_vm_eval_expression(PetalVM* vm, VMScope* scope, const Expression* expression) {
    switch (expression->kind) {
    case EXPRESSION_KIND_NUMBER_LITERAL: {
        return (VMValue){.kind = VM_VALUE_KIND_NUMBER, .number = expression->number_literal};
    }

    case EXPRESSION_KIND_FUNCTION_CALL: {
        VMScope call_scope = {0};
        vm_scope_init(&call_scope, vm->allocator);

        if (!petal_vm_get_and_call_function(vm, &call_scope, &expression->function_call)) {
            return (VMValue){.kind = VM_VALUE_NOTHING};
        }

        return call_scope.return_value;
    }

    case EXPRESSION_KIND_STRING_LITERAL: {
        return (VMValue){.kind = VM_VALUE_KIND_STRING, .string = expression->string};
    }

    case EXPRESSION_KIND_IDENTIFIER_REFERENCE: {
        for (size_t i = 0; i < scope->variables.length; i++) {
            VMVariable* variable = &scope->variables.data[i];
            if (!string_buffer_equals(&variable->name, &expression->string)) {
                continue;
            }

            return variable->value;
        }

        log_error(
            "vm: could not find variable with name '%.*s'",
            (int)expression->string.length,
            expression->string.data
        );

        return (VMValue){.kind = VM_VALUE_NOTHING};
    }
    }

    return (VMValue){.kind = VM_VALUE_NOTHING};
}

const FunctionDeclarationStatement* petal_vm_get_function(PetalVM* vm, const StringBuffer* function_name) {
    for (size_t i = 0; i < vm->functions.length; i++) {
        const FunctionDeclarationStatement* function = &vm->functions.data[i];

        if (!string_buffer_equals(function_name, &function->name)) {
            continue;
        }

        return function;
    }

    return NULL;
}

bool petal_vm_call_function(
    PetalVM* vm,
    VMScope* scope,
    const FunctionDeclarationStatement* function,
    const ExpressionArray* arguments
) {
    assert(function != NULL && "petal_vm_call_function: function cannot be null");
    assert(arguments != NULL && "petal_vm_call_function: arguments cannot be null");

    if (function->parameters.length != arguments->length) {
        log_error(
            "function '%.*s' requires %zu parameter(s) but %zu argument(s) were provided",
            (int)function->name.length,
            function->name.data,
            function->parameters.length,
            arguments->length
        );

        return false;
    }

    for (size_t i = 0; i < function->parameters.length; i++) {
        const FunctionParameter* parameter = &function->parameters.data[i];
        const Expression* argument = arguments->data[i];

        const VMVariable variable = (VMVariable){
            .name = parameter->name,
            .value = petal_vm_eval_expression(vm, scope, argument),
        };

        vm_variable_array_append(&scope->variables, variable);
    }

    for (size_t i = 0; i < function->body.length; i++) {
        if (!petal_vm_exec_statement(vm, scope, function->body.data[i])) {
            return false;
        }

        if (scope->stop_execution) {
            break;
        }
    }

    return true;
}

bool petal_vm_get_and_call_function(PetalVM* vm, VMScope* scope, const FunctionCall* call) {
    const VMBuiltinFunction* builtin_function = petal_vm_get_builtin_function(vm, call);
    if (builtin_function) {
        VMValueArray arguments = {0};
        vm_value_array_init(&arguments, vm->allocator);

        for (size_t i = 0; i < call->arguments.length; i++) {
            vm_value_array_append(&arguments, petal_vm_eval_expression(vm, scope, call->arguments.data[i]));
        }

        scope->return_value = builtin_function->handler(vm, &arguments);
        return true;
    }

    const FunctionDeclarationStatement* function = petal_vm_get_function(vm, &call->name);
    if (!function) {
        log_error("vm: could not find function with name '%.*s'", (int)call->name.length, call->name.data);
        return false;
    }

    return petal_vm_call_function(vm, scope, function, &call->arguments);
}

const VMBuiltinFunction* petal_vm_get_builtin_function(PetalVM* vm, const FunctionCall* call) {
    (void)vm;

    const size_t number_of_builtins = sizeof(builtin_functions) / sizeof(VMBuiltinFunction);

    for (size_t i = 0; i < number_of_builtins; i++) {
        const VMBuiltinFunction* function = &builtin_functions[i];
        if (!string_buffer_equals_cstr(&call->name, function->name)) {
            continue;
        }

        return function;
    }

    return NULL;
}
