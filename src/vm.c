#include "vm.h"
#include "allocator.h"
#include "array.h"
#include "ast_statement.h"
#include "ast_type.h"
#include "logger.h"
#include "vm_value.h"
#include <assert.h>

IMPLEMENT_ARRAY_TYPE(FunctionDeclarationArray, function_declaration_array, FunctionDeclarationStatement)

/**
 * Attempts to find a declaration for the main method in the VM's current state, returning NULL if one could not be
 * found.
 */
const FunctionDeclarationStatement* petal_vm_get_main_function(const PetalVM* vm);

/**
 * Attempts to execute the provided statement, updating the VM state if neccessary.
 */
bool petal_vm_exec_statement(PetalVM* vm, VMScope* scope, Statement* statement);

/**
 * Evaluates the provided expression. If a nothing value is returned, the evaluation failed.
 */
VMValue petal_vm_eval_expression(PetalVM* vm, VMScope* scope, const Expression* expression);

void petal_vm_init(PetalVM* vm, Allocator* allocator, const StatementArray* statements) {
    assert(vm != NULL && "NULL PetalVM passed to petal_vm_init");
    assert(allocator != NULL && "NULL Allocator passed to petal_vm_init");
    assert(statements != NULL && "NULL StatementArray passed to petal_vm_init");

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

    const FunctionDeclarationStatement* main_function = petal_vm_get_main_function(vm);
    if (!main_function) {
        return false;
    }

    VMScope scope = {.continue_execution = true};

    for (size_t i = 0; i < main_function->body.length; i++) {
        if (!petal_vm_exec_statement(vm, &scope, main_function->body.data[i])) {
            return false;
        }

        if (!scope.continue_execution) {
            break;
        }
    }

    // The scope should have a number return value.
    if (scope.return_value.kind != VM_VALUE_KIND_NUMBER) {
        log_error("vm: main function did not return an integer");
        return false;
    }

    vm->state.exit_code = (int)scope.return_value.number;
    return true;
}

bool petal_vm_exec_statement(PetalVM* vm, VMScope* scope, Statement* statement) {
    (void)vm;

    switch (statement->kind) {
    case STATEMENT_KIND_RETURN: {
        scope->continue_execution = false;

        if (statement->return_.value) {
            scope->return_value = petal_vm_eval_expression(vm, scope, statement->return_.value);
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
    (void)vm;
    (void)scope;

    switch (expression->kind) {
    case EXPRESSION_KIND_NUMBER_LITERAL:
        return (VMValue){.kind = VM_VALUE_KIND_NUMBER, .number = expression->number_literal};
        break;
    }

    return (VMValue){.kind = VM_VALUE_NOTHING};
}

const FunctionDeclarationStatement* petal_vm_get_main_function(const PetalVM* vm) {
    for (size_t i = 0; i < vm->functions.length; i++) {
        const FunctionDeclarationStatement* function = &vm->functions.data[i];
        if (!string_buffer_equals_cstr(&function->name, "main")) {
            continue;
        }

        // TODO: 32-bit integer type.
        if (function->return_type.kind != TYPE_KIND_UNKNOWN ||
            !string_buffer_equals_cstr(&function->return_type.type_name, "i32")) {
            log_error("vm: return type of main function must be i32");
            return NULL;
        }

        if (function->parameters.length != 0) {
            log_error("vm: main function parameters are not supported yet");
            return NULL;
        }

        return function;
    }

    log_error(
        "vm: could not find main function declaration. ensure that you have a function named `main` with an `i32` "
        "return type"
    );

    return NULL;
}
