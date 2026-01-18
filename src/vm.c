#include "vm.h"
#include "allocator.h"
#include "array.h"
#include "ast_statement.h"
#include "logger.h"
#include "vm_value.h"
#include <assert.h>

IMPLEMENT_ARRAY_TYPE(FunctionDeclarationArray, function_declaration_array, FunctionDeclarationStatement)

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
 * Attempts to get and call the function using the information in the [FunctionCall].
 */
bool petal_vm_get_and_call_function(PetalVM* vm, VMScope* scope, const FunctionCall* call);

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

    if (!petal_vm_get_and_call_function(vm, &scope, &main_function_call)) {
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
        if (!petal_vm_get_and_call_function(vm, &(VMScope){0}, &statement->function_call)) {
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
    (void)scope; // TODO: scope inheritance

    switch (expression->kind) {
    case EXPRESSION_KIND_NUMBER_LITERAL: {
        return (VMValue){.kind = VM_VALUE_KIND_NUMBER, .number = expression->number_literal};
    }

    case EXPRESSION_KIND_FUNCTION_CALL: {
        VMScope call_scope = {0};

        if (!petal_vm_get_and_call_function(vm, &call_scope, &expression->function_call)) {
            return (VMValue){.kind = VM_VALUE_NOTHING};
        }

        return call_scope.return_value;
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
    const FunctionDeclarationStatement* function = petal_vm_get_function(vm, &call->name);
    if (!function) {
        return false;
    }

    return petal_vm_call_function(vm, scope, function, &call->arguments);
}
