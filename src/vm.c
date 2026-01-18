#include "vm.h"
#include "allocator.h"
#include "array.h"
#include "ast_statement.h"
#include "ast_type.h"
#include "logger.h"
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
bool petal_vm_exec_statement(PetalVM* vm, Statement* statement);

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
    const FunctionDeclarationStatement* main_function = petal_vm_get_main_function(vm);
    if (!main_function) {
        return false;
    }

    for (size_t i = 0; i < main_function->body.length; i++) {
        if (!petal_vm_exec_statement(vm, main_function->body.data[i])) {
            return false;
        }
    }

    return true;
}

bool petal_vm_exec_statement(PetalVM* vm, Statement* statement) {
    switch (statement->kind) {
    case STATEMENT_KIND_RETURN: {
        // TODO: Support creating a scope and returning values from that scope instead of assuming that we're in the
        // main function.
        if (!statement->return_.value) {
            break;
        }

        if (statement->return_.value->kind != EXPRESSION_KIND_NUMBER_LITERAL) {
            log_error("returning values other than number literals is not supported yet");
            return false;
        }

        vm->state.exit_code = (size_t)statement->return_.value->number_literal;
        break;
    }

    case STATEMENT_KIND_FUNCTION_DECLARATION:
        log_error("encountered a top-level statement kind while executing scope-based statement?");
        return false;
    }

    return true;
}

const FunctionDeclarationStatement* petal_vm_get_main_function(const PetalVM* vm) {
    for (size_t i = 0; i < vm->functions.length; i++) {
        const FunctionDeclarationStatement* function = &vm->functions.data[i];
        if (!string_buffer_equals_cstr(&function->name, "main")) {
            continue;
        }

        // TODO: 32-bit integer type.
        if (function->return_type.kind != TYPE_KIND_UNKNOWN &&
            !string_buffer_equals_cstr(&function->return_type.type_name, "i32")) {
            log_error("return type of main function must be i32");
            return NULL;
        }

        if (function->parameters.length != 0) {
            log_error("main function parameters are not supported yet");
            return NULL;
        }

        return function;
    }

    log_error("failed to find main function");
    return NULL;
}
