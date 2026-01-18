#include "vm.h"
#include "allocator.h"
#include "array.h"
#include "ast_statement.h"
#include "logger.h"
#include "vm_value.h"
#include <assert.h>

IMPLEMENT_ARRAY_TYPE(FunctionDeclarationArray, function_declaration_array, FunctionDeclarationStatement)

/**
 * Attempts to execute the provided statement, updating the VM state if neccessary.
 */
bool petal_vm_exec_statement(PetalVM* vm, VMScope* scope, Statement* statement);

/**
 * Evaluates the provided expression. If a nothing value is returned, the evaluation failed.
 */
VMValue petal_vm_eval_expression(PetalVM* vm, VMScope* scope, const Expression* expression);

/**
 * Attempts to call the function with the provided name and arguments.
 */
bool petal_vm_call_function(PetalVM* vm, VMScope* scope, const StringBuffer* name, const ExpressionArray* arguments);

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

    StringBuffer main_function_name = {0};
    string_buffer_init_from_cstr(&main_function_name, vm->allocator, "main");

    VMScope scope = {0};

    if (!petal_vm_call_function(vm, &scope, &main_function_name, &(ExpressionArray){0})) {
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
        const FunctionCall function_call = statement->function_call;

        VMScope call_scope = {0};
        if (!petal_vm_call_function(vm, &call_scope, &function_call.name, &function_call.arguments)) {
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
    case EXPRESSION_KIND_NUMBER_LITERAL:
        return (VMValue){.kind = VM_VALUE_KIND_NUMBER, .number = expression->number_literal};
        break;

    case EXPRESSION_KIND_FUNCTION_CALL: {
        const FunctionCall function_call = expression->function_call;

        VMScope call_scope = {0};
        if (!petal_vm_call_function(vm, &call_scope, &function_call.name, &function_call.arguments)) {
            return (VMValue){.kind = VM_VALUE_NOTHING};
        }

        return call_scope.return_value;
    }
    }

    return (VMValue){.kind = VM_VALUE_NOTHING};
}

bool petal_vm_call_function(PetalVM* vm, VMScope* scope, const StringBuffer* name, const ExpressionArray* arguments) {
    (void)arguments;
    assert(arguments->length == 0 && "petal_vm_call_function: arguments are not supported");

    FunctionDeclarationStatement* function = NULL;

    for (size_t i = 0; i < vm->functions.length; i++) {
        FunctionDeclarationStatement* current_function = &vm->functions.data[i];
        if (!string_buffer_equals(name, &current_function->name)) {
            continue;
        }

        function = current_function;
    }

    if (!function) {
        log_error("vm: could not find function with name '%.*s'", (int)name->length, name->data);
        return false;
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
