#include "typechecker.h"
#include "ast/node.h"
#include "ast/node/binary_operation.h"
#include "ast/node/function_call.h"
#include "ast/node/function_declaration.h"
#include "ast/node/identifier_reference.h"
#include "ast/node/number_literal.h"
#include "ast/node/return.h"
#include "ast/node/variable_declaration.h"
#include "ast/node/variable_reassignment.h"
#include "core/diagnostic.h"
#include "core/type/reference.h"
#include "core/type/type.h"
#include "core/type/unresolved.h"
#include "core/type/value.h"
#include "typechecker/context.h"
#include "typechecker/declared_function.h"
#include "typechecker/declared_variable.h"
#include "util/defer.h"
#include "util/format.h"
#include "util/logger.h"
#include "util/vector.h"
#include <stdio.h>
#include <string.h>

// Forward declarations:
bool typechecker_check_statement(Typechecker* typechecker, Node* node);
bool typechecker_check_function_declaration(Typechecker* typechecker, FunctionDeclarationNode* node);
bool typechecker_check_variable_declaration(Typechecker* typechecker, VariableDeclarationNode* node);
bool typechecker_check_return(Typechecker* typechecker, ReturnNode* node);
bool typechecker_check_variable_reassignment(Typechecker* typechecker, VariableReassignmentNode* node);

Type* typechecker_check_expression(Typechecker* typechecker, Node* node);
Type* typechecker_check_number_literal(Typechecker* typechecker, NumberLiteralNode* node);
Type* typechecker_check_identifier_reference(Typechecker* typechecker, IdentifierReferenceNode* node);
Type* typechecker_check_binary_operation(Typechecker* typechecker, BinaryOperationNode* node);
Type* typechecker_check_function_call(Typechecker* typechecker, FunctionCallNode* node);

// Resolves a type.
// If the type could not be resolved, nullptr is returned.
Type* typechecker_resolve_type(Typechecker* typechecker, Type** type_reference);

Typechecker typechecker_create(NodeVector* nodes, DiagnosticVector* diagnostics) {
    return (Typechecker){
        .nodes = nodes,
        .diagnostics = diagnostics,
        .context = (TypecheckerContext){},
        .declared_functions = vector_create(),
    };
}

bool typechecker_check(Typechecker* typechecker) {
    if (!vector_initialize(typechecker->declared_functions, 1)) {
        diagnostic_create((Position){}, "failed to initialize typechecker");
        return false;
    }

    for (size_t i = 0; i < typechecker->nodes->length; i++) {
        auto node = vector_get(typechecker->nodes, i);
        if (!typechecker_check_statement(typechecker, node)) {
            return false;
        }
    }

    return true;
}

void typechecker_destroy(Typechecker* typechecker) {
    free(typechecker->declared_functions.items);
}

bool typechecker_check_statement(Typechecker* typechecker, Node* node) {
    switch (node->kind) {
    case NODE_KIND_FUNCTION_DECLARATION:
        return typechecker_check_function_declaration(typechecker, (FunctionDeclarationNode*)node);

    case NODE_KIND_VARIABLE_DECLARATION:
        return typechecker_check_variable_declaration(typechecker, (VariableDeclarationNode*)node);

    case NODE_KIND_RETURN:
        return typechecker_check_return(typechecker, (ReturnNode*)node);

    case NODE_KIND_FUNCTION_CALL:
        return typechecker_check_function_call(typechecker, (FunctionCallNode*)node);

    case NODE_KIND_VARIABLE_REASSIGNMENT:
        return typechecker_check_variable_reassignment(typechecker, (VariableReassignmentNode*)node);

    default:
        auto node_string defer(free_str) = node_to_string(node);
        vector_append(
            typechecker->diagnostics,
            diagnostic_create(node->position, format_string("unable to type-check node: '%s'", node_string))
        );

        return false;
    }
}

bool typechecker_check_function_declaration(Typechecker* typechecker, FunctionDeclarationNode* node) {
    // The function's return type must be resolvable.
    auto return_type = typechecker_resolve_type(typechecker, &node->return_type);
    if (!return_type) {
        return false;
    }

    // Before typechecking the nodes, we can set the context's return type to the function's return type.
    if (!typechecker_context_initialize(&typechecker->context, return_type)) {
        vector_append(
            typechecker->diagnostics,
            diagnostic_create(
                node->header.position,
                "internal typechecker error: failed to initialize typechecker context"
            )
        );

        return false;
    }

    // The function's parameters must also have valid types.
    for (size_t i = 0; i < node->parameters.length; i++) {
        auto parameter = vector_get_ref(&node->parameters, i);
        if (!typechecker_resolve_type(typechecker, &parameter->value_type)) {
            typechecker_context_destroy(&typechecker->context);
            return false;
        }

        // We can treat the function's parameters as declared variables within this context.
        auto variable = (DeclaredVariable){.name = parameter->name, .type = parameter->value_type};
        vector_append(&typechecker->context.declared_variables, variable);
    }

    // If the return type is OK, and this is a non-extern function we can type check the function's body.
    if (!(node->modifiers & FUNCTION_MODIFIER_EXTERN)) {
        for (size_t i = 0; i < node->body.length; i++) {
            auto body_node = vector_get(&node->body, i);
            if (!typechecker_check_statement(typechecker, body_node)) {
                typechecker_context_destroy(&typechecker->context);
                return false;
            }
        }
    }

    // We can record this function as a declared function.
    auto declared_function = (DeclaredFunction){
        .name = node->name,
        .return_type = node->return_type,
        .parameters = &node->parameters,
    };

    vector_append(&typechecker->declared_functions, declared_function);
    typechecker_context_destroy(&typechecker->context);
    return true;
}

bool typechecker_check_variable_declaration(Typechecker* typechecker, VariableDeclarationNode* node) {
    // If the current context does not have a declared variable vector, let's say that they're not allowed here.
    if (typechecker->context.declared_variables.capacity == 0) {
        auto position = node->type->position;
        position.length = (node->value->position.column + node->value->position.length) - node->type->position.column;

        vector_append(
            typechecker->diagnostics,
            diagnostic_create(position, format_string("variable declarations are not allowed here"))
        );

        return false;
    }

    // The variable's expected type must be resolvable.
    auto variable_type = typechecker_resolve_type(typechecker, &node->type);
    if (!variable_type) {
        return false;
    }

    // The variable's initial value must be resolvable.
    auto value_type = typechecker_check_expression(typechecker, node->value);
    if (!value_type) {
        return false;
    }

    // The type of the variable must be the same as the value.
    if (!type_equals(variable_type, value_type)) {
        auto variable_type_string defer(free_str) = type_to_string((Type*)variable_type);
        auto value_type_string defer(free_str) = type_to_string((Type*)value_type);

        vector_append(
            typechecker->diagnostics,
            diagnostic_create(
                value_type->position,
                format_string("expected type '%s', but got '%s'", variable_type_string, value_type_string)
            )
        );

        return false;
    }

    // The types are matching, we can record this as a declared variable.
    vector_append(&typechecker->context.declared_variables, declared_variable_create(node->name, variable_type));
    return true;
}

bool typechecker_check_return(Typechecker* typechecker, ReturnNode* node) {
    // The current context must have an expected return type.
    auto expected_return_type = typechecker->context.expected_return_type;
    if (!expected_return_type) {
        vector_append(
            typechecker->diagnostics,
            diagnostic_create(
                node->header.position,
                format_string("internal typechecker error: current context does not have an expected return type. "
                              "unable to typecheck node.")
            )
        );

        return false;
    }

    // If this return statement has no value, there is no type-checking to do.
    if (!node->return_value) {
        // TODO: Ensure that the current function has a return value of `void`.
        return true;
    }

    // The return value's type must be resolvable.
    auto value_type = typechecker_check_expression(typechecker, node->return_value);
    if (!value_type) {
        return false;
    }

    if (!type_equals(expected_return_type, value_type)) {
        auto expected_return_type_string defer(free_str) = type_to_string(expected_return_type);
        auto value_type_string defer(free_str) = type_to_string(value_type);

        vector_append(
            typechecker->diagnostics,
            diagnostic_create(
                node->header.position,
                format_string(
                    "unable to return '%s' from function returning '%s'",
                    value_type_string,
                    expected_return_type_string
                )
            )
        );

        return false;
    }

    return true;
}

bool typechecker_check_variable_reassignment(Typechecker* typechecker, VariableReassignmentNode* node) {
    // The variable being re-assigned must exist.
    auto variable = declared_variable_find_by_name(typechecker->context.declared_variables, node->name);
    if (!variable) {
        return false;
    }

    // The value's type must be resolvable.
    auto value_type = typechecker_check_expression(typechecker, node->value);
    if (!value_type) {
        return false;
    }

    // The type of the variable must match the values type.
    if (!type_equals(variable->type, value_type)) {
        auto variable_type_string defer(free_str) = type_to_string((Type*)variable->type);
        auto value_type_string defer(free_str) = type_to_string((Type*)value_type);

        vector_append(
            typechecker->diagnostics,
            diagnostic_create(
                value_type->position,
                format_string("expected type '%s', but got '%s'", variable_type_string, value_type_string)
            )
        );

        return false;
    }

    return true;
}

Type* typechecker_check_expression(Typechecker* typechecker, Node* node) {
    switch (node->kind) {
    case NODE_KIND_NUMBER_LITERAL:
        return typechecker_check_number_literal(typechecker, (NumberLiteralNode*)node);

    case NODE_KIND_IDENTIFIER_REFERENCE:
        return typechecker_check_identifier_reference(typechecker, (IdentifierReferenceNode*)node);

    case NODE_KIND_BINARY_OPERATION:
        return typechecker_check_binary_operation(typechecker, (BinaryOperationNode*)node);

    case NODE_KIND_FUNCTION_CALL:
        return typechecker_check_function_call(typechecker, (FunctionCallNode*)node);

    default:
        auto node_string defer(free_str) = node_to_string(node);
        vector_append(
            typechecker->diagnostics,
            diagnostic_create(node->position, format_string("unable to type-check node: '%s'", node_string))
        );

        return nullptr;
    }
}

Type* typechecker_check_number_literal(Typechecker* typechecker, NumberLiteralNode* node) {
    (void)typechecker;

    // All integer literals are i32 and all float literals are f64 for now.
    Type* type;
    if (node->is_float) {
        type = (Type*)value_type_create(node->header.position, VALUE_TYPE_KIND_F64);
    } else {
        type = (Type*)value_type_create(node->header.position, VALUE_TYPE_KIND_I32);
    }

    node->type = type;
    return type;
}

Type* typechecker_check_identifier_reference(Typechecker* typechecker, IdentifierReferenceNode* node) {
    // The identifier must be resolvable to a variable declaration.
    auto variable = declared_variable_find_by_name(typechecker->context.declared_variables, node->identifier);
    if (!variable) {
        vector_append(
            typechecker->diagnostics,
            diagnostic_create(node->header.position, format_string("undefined variable: '%s'", node->identifier))
        );

        return nullptr;
    }

    // If this identifier is not being passed as a reference, there is nothing else to do.
    if (!node->value_type || node->value_type->kind != TYPE_KIND_REFERENCE) {
        return variable->type;
    }

    // The type is a reference to the variable's value type.
    node->value_type = (Type*)reference_type_create(variable->type->position, variable->type);
    return node->value_type;
}

Type* typechecker_check_binary_operation(Typechecker* typechecker, BinaryOperationNode* node) {
    // The value on the left side must have a resolvable type.
    auto left_type = typechecker_check_expression(typechecker, node->left);
    if (!left_type) {
        return nullptr;
    }

    // The value on the right must have a resolvable type.
    auto right_type = typechecker_check_expression(typechecker, node->right);
    if (!right_type) {
        return nullptr;
    }

    // The types must be the same.
    if (!type_equals(left_type, right_type)) {
        auto left_type_string defer(free_str) = type_to_string(left_type);
        auto right_type_string defer(free_str) = type_to_string(right_type);

        vector_append(
            typechecker->diagnostics,
            diagnostic_create(
                node->header.position,
                format_string(
                    "unable to perform operation '%s' between '%s' and '%s'",
                    operator_to_string(node->operator),
                    left_type_string,
                    right_type_string
                )
            )
        );

        return nullptr;
    }

    return left_type;
}

Type* typechecker_check_function_call(Typechecker* typechecker, FunctionCallNode* node) {
    auto function = declared_function_find_by_name(typechecker->declared_functions, node->function_name);
    if (!function) {
        vector_append(
            typechecker->diagnostics,
            diagnostic_create(node->header.position, format_string("undeclared function: '%s'", node->function_name))
        );

        return nullptr;
    }

    LOG_DEBUG("typechecker", "checking function call for '%s'", node->function_name);

    // The number of arguments must match the number of parameters expected by the function.
    if (node->arguments.length != function->parameters->length) {
        vector_append(
            typechecker->diagnostics,
            diagnostic_create(
                node->header.position,
                format_string(
                    "function '%s' has %zu parameter(s) but %zu argument(s) were passed",
                    node->function_name,
                    function->parameters->length,
                    node->arguments.length
                )
            )
        );

        return nullptr;
    }

    // The types of the arguments must match the parameters.
    for (size_t i = 0; i < node->arguments.length; i++) {
        auto argument = vector_get(&node->arguments, i);
        auto parameter = vector_get(function->parameters, i);

        // The argument must have a resolvable type.
        auto argument_type = typechecker_check_expression(typechecker, argument);
        if (!argument_type) {
            return nullptr;
        }

        // If the argument's type does not match the defined parameter, throw an error.s
        if (!type_equals(argument_type, parameter.value_type)) {
            auto argument_type_string defer(free_str) = type_to_string(argument_type);
            auto parameter_type_string defer(free_str) = type_to_string(parameter.value_type);

            vector_append(
                typechecker->diagnostics,
                diagnostic_create(
                    argument->position,
                    format_string(
                        "unable to pass argument of type '%s' to function with parameter of type '%s'",
                        argument_type_string,
                        parameter_type_string
                    )
                )
            );

            return nullptr;
        }
    }

    // The type of this function call is the function's return type.
    return function->return_type;
}

Type* typechecker_resolve_type(Typechecker* typechecker, Type** type_reference) {
    auto type = *type_reference;

    // If the type is already resolved, we don't need to do anything.
    if (type->kind == TYPE_KIND_VALUE) {
        return type;
    }

    // If this is a reference type, we need to resolve the inner type.
    if (type->kind == TYPE_KIND_REFERENCE) {
        auto reference_type = (ReferenceType*)type;
        auto resolved_type = typechecker_resolve_type(typechecker, &reference_type->referenced_type);
        if (!resolved_type) {
            return nullptr;
        }

        return type;
    }

    // This type is unresolved.
    auto unresolved_type = (UnresolvedType*)type;

    // In order to resolve it, we need to see if it is a valid "value" type.
    auto value_type_kind = value_type_kind_from_string(unresolved_type->name);
    if (value_type_kind == VALUE_TYPE_KIND_INVALID) {
        vector_append(
            typechecker->diagnostics,
            diagnostic_create(type->position, format_string("unknown type: '%s'", unresolved_type->name))
        );

        return nullptr;
    }

    // The type has been resolved, we can now assign the original type to it.
    auto resolved_type = (Type*)value_type_create(type->position, value_type_kind);
    *type_reference = resolved_type;

    // The original type is no longer needed.
    type_destroy(type);

    return resolved_type;
}
