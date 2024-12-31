#include "typechecker.h"
#include "../ast/node/binary_operation.h"
#include "../ast/node/boolean_literal.h"
#include "../ast/node/force_unwrap.h"
#include "../ast/node/function_call.h"
#include "../ast/node/function_declaration.h"
#include "../ast/node/identifier_reference.h"
#include "../ast/node/number_literal.h"
#include "../ast/node/return.h"
#include "../ast/node/string_literal.h"
#include "../ast/node/type_alias_declaration.h"
#include "../ast/node/variable_declaration.h"
#include "../ast/node/variable_reassignment.h"
#include "declared_function.h"
#include "declared_variable.h"
#include "type_alias.h"
#include <math.h>
#include <stdbool.h>

// Forward declarations:
bool typechecker_check(Typechecker* typechecker, NodeStream* node_stream, ResolvedType* return_type);

ResolvedType* typechecker_resolve_type(Typechecker* typechecker, Type** type_reference, Position position);

bool typechecker_check_statement(Typechecker* typechecker, Node* node, ResolvedType* return_type);
bool typechecker_check_function_declaration(Typechecker* typechecker, FunctionDeclarationNode* node);
bool typechecker_check_variable_declaration(Typechecker* typechecker, VariableDeclarationNode* node);
bool typechecker_check_return(Typechecker* typechecker, ReturnNode* node, ResolvedType* return_type);
bool typechecker_check_type_alias_declaration(Typechecker* typechecker, TypeAliasDeclarationNode* node);
bool typechecker_check_variable_reassignment(Typechecker* typechecker, VariableReassignmentNode* node);

ResolvedType* typechecker_check_value(Typechecker* typechecker, Node* value, ResolvedType* expected_type);
ResolvedType* typechecker_check_number_literal(
    Typechecker* typechecker,
    NumberLiteralNode* node,
    ResolvedType* expected_type
);
ResolvedType* typechecker_check_string_literal(Typechecker* typechecker, StringLiteralNode* node);
ResolvedType* typechecker_check_boolean_literal(Typechecker* typechecker, BooleanLiteralNode* node);
ResolvedType* typechecker_check_identifier_reference(Typechecker* typechecker, IdentifierReferenceNode* node);
ResolvedType* typechecker_check_binary_operation(
    Typechecker* typechecker,
    BinaryOperationNode* node,
    ResolvedType* expected_type
);
ResolvedType* typechecker_check_function_call(Typechecker* typechecker, FunctionCallNode* node);
ResolvedType* typechecker_check_force_unwrap(
    Typechecker* typechecker,
    ForceUnwrapNode* node,
    ResolvedType* expected_type
);

Typechecker typechecker_create() {
    DiagnosticStream diagnostics;
    diagnostic_stream_initialize(&diagnostics, 1);

    DeclaredFunctions functions;
    declared_functions_initialize(&functions, 1);

    DeclaredVariables variables;
    declared_variables_initialize(&variables, 1);

    TypeAliases type_aliases;
    type_aliases_initialize(&type_aliases, 1);

    return (Typechecker){diagnostics, functions, variables, type_aliases};
}

void typechecker_run(Typechecker* typechecker, NodeStream* node_stream) {
    typechecker_check(typechecker, node_stream, 0);
}

bool typechecker_check(Typechecker* typechecker, NodeStream* node_stream, ResolvedType* return_type) {
    bool success = true;

    for (size_t i = 0; i < node_stream->length; i++) {
        Node* node = node_stream->data[i];
        if (!typechecker_check_statement(typechecker, node, return_type)) {
            success = false;
        }
    }

    return success;
}

ResolvedType* typechecker_resolve_type(Typechecker* typechecker, Type** type_reference, Position position) {
    // If the type is already resolved, just return it.
    Type* type = *type_reference;
    if (type->is_resolved) {
        return (ResolvedType*)type;
    }

    // This is an unresolved type.
    UnresolvedType* unresolved_type = (UnresolvedType*)type;

    // The name may be referring to a type alias.
    TypeAlias* alias = type_aliases_find_by_name(typechecker->type_aliases, unresolved_type->name);
    if (alias) {
        *type_reference = (Type*)alias->type;
        return alias->type;
    }

    // There is no type alias for the type, attempt to resolve a valid type-kind from the name.
    TypeKind resolved_type_kind = type_kind_from_string(unresolved_type->name);
    if (resolved_type_kind == TYPE_KIND_INVALID) {
        diagnostic_stream_push(&typechecker->diagnostics, position, true, "unknown type: '%s'", unresolved_type->name);
        return 0;
    }

    ResolvedType* resolved_type =
        type_create_resolved(unresolved_type->is_optional, unresolved_type->is_reference, resolved_type_kind);

    // Now that we have resolved the type, we can update the original type pointer to be the new type.
    *type_reference = (Type*)resolved_type;
    return resolved_type;
}

bool typechecker_check_statement(Typechecker* typechecker, Node* node, ResolvedType* return_type) {
    switch (node->node_type) {
    case NODE_FUNCTION_DECLARATION:
        return typechecker_check_function_declaration(typechecker, (FunctionDeclarationNode*)node);

    case NODE_VARIABLE_DECLARATION:
        return typechecker_check_variable_declaration(typechecker, (VariableDeclarationNode*)node);

    case NODE_RETURN:
        return typechecker_check_return(typechecker, (ReturnNode*)node, return_type);

    case NODE_FUNCTION_CALL: {
        ResolvedType* return_type = typechecker_check_function_call(typechecker, (FunctionCallNode*)node);
        if (!return_type) {
            return false;
        }

        return true;
    }

    case NODE_TYPE_ALIAS_DECLARATION:
        return typechecker_check_type_alias_declaration(typechecker, (TypeAliasDeclarationNode*)node);

    case NODE_VARIABLE_REASSIGNMENT:
        return typechecker_check_variable_reassignment(typechecker, (VariableReassignmentNode*)node);

    case NODE_IMPORT:
        return true; // ignore

    default: {
        diagnostic_stream_push(
            &typechecker->diagnostics,
            node->position,
            true,
            "unable to type-check node: '%s'",
            node_to_string(node)
        );

        return false;
    }
    }
}

bool typechecker_check_function_declaration(Typechecker* typechecker, FunctionDeclarationNode* node) {
    ResolvedType* return_type = typechecker_resolve_type(typechecker, &node->return_type, node->position);
    if (!return_type) {
        return false;
    }

    for (size_t i = 0; i < node->parameters.length; i++) {
        Parameter* parameter = &node->parameters.data[i];
        ResolvedType* parameter_type = typechecker_resolve_type(typechecker, &parameter->type, node->position);
        if (!parameter_type) {
            return 0;
        }
    }

    declared_functions_append(
        &typechecker->functions,
        (DeclaredFunction){.name = node->name, .return_type = return_type, .parameters = node->parameters}
    );

    if (node->is_external) {
        // External functions have nothing to typecheck at the moment.
        LOG_DEBUG("typechecker", "skipping external function: '%s'", node->name);
        return true;
    }

    // If this is not an external function, and there is no function body, that should not be supported.
    if (!node->function_body) {
        diagnostic_stream_push(
            &typechecker->diagnostics,
            node->position,
            true,
            "non-external function '%s' has no function body",
            node->name
        );

        return false;
    }

    LOG_DEBUG("typechecker", "typechecking function: '%s'", node->name);

    // Before doing any typechecking within the body, we should treat this as a new scope.
    declared_variables_destroy(&typechecker->variables);
    declared_variables_initialize(&typechecker->variables, 1);

    // Function parameters are *technicaly* local values.
    // FIXME: This feels a little bit wrong, but it's fine for now?
    for (size_t i = 0; i < node->parameters.length; i++) {
        Parameter parameter = node->parameters.data[i];

        // Casting parameter.type to ResolvedType is safe here, we confirmed above that the type has been resolved.
        declared_variables_append(
            &typechecker->variables,
            (DeclaredVariable){.name = parameter.name, .type = (ResolvedType*)parameter.type}
        );
    }

    // Function declarations don't have much to typecheck, their parameters should already have types defined,
    // so all we have to do is verify that the function body is OK.
    return typechecker_check(typechecker, &node->function_body->body, return_type);
}

bool typechecker_check_variable_declaration(Typechecker* typechecker, VariableDeclarationNode* node) {
    // The variable's type must be a resolvable type.
    ResolvedType* variable_type = typechecker_resolve_type(typechecker, &node->type, node->position);
    if (!variable_type) {
        return false;
    }
    // Record this as a declared variable within this scope.
    declared_variables_append(&typechecker->variables, (DeclaredVariable){.name = node->name, .type = variable_type});

    // If the declaration does not have a value, it must be of an optional type.
    if (!node->value && !variable_type->is_optional) {
        diagnostic_stream_push(
            &typechecker->diagnostics,
            node->position,
            true,
            "variable '%s' does not have an initial value",
            node->name
        );

        return false;
    }

    // If there is no value, there is nothing left to check.
    if (!node->value) {
        return true;
    }

    // A variable declaration always has an expected type.
    // If the value does not match the expected type, we must throw an error.
    ResolvedType* value_type = typechecker_check_value(typechecker, node->value, variable_type);
    if (!value_type) {
        return false;
    }

    // Allow non-null values to be assigned to nullable variables.
    // Anything else is a type mismatch.
    if (variable_type->is_reference != value_type->is_reference || variable_type->kind != value_type->kind ||
        (!variable_type->is_optional && value_type->is_optional)) {
        diagnostic_stream_push(
            &typechecker->diagnostics,
            node->value->position,
            true,
            "unable to assign value of type '%s' to type '%s'",
            type_to_string((Type*)value_type),
            type_to_string((Type*)variable_type)
        );
        return false;
    }

    return true;
}

bool typechecker_check_return(Typechecker* typechecker, ReturnNode* node, ResolvedType* return_type) {
    if (!return_type) {
        diagnostic_stream_push(
            &typechecker->diagnostics,
            node->position,
            true,
            "unable to type check return statement, current function has no known return type?"
        );
        return false;
    }

    // The value within a return node must match the return type of the function.
    ResolvedType* value_type = typechecker_check_value(typechecker, node->value, return_type);
    if (!value_type) {
        return false;
    }

    if (!type_equal((Type*)return_type, (Type*)value_type)) {
        // These types are not matching!
        diagnostic_stream_push(
            &typechecker->diagnostics,
            node->value->position,
            true,
            "type '%s' cannot be returned from a function with return type '%s'",
            type_to_string((Type*)value_type),
            type_to_string((Type*)return_type)
        );
        return false;
    }

    return true;
}

bool typechecker_check_type_alias_declaration(Typechecker* typechecker, TypeAliasDeclarationNode* node) {
    // The type being aliased must be resolvable.
    ResolvedType* aliased_type = typechecker_resolve_type(typechecker, &node->type, node->position);
    if (!aliased_type) {
        return false;
    }

    // The type alias can now be recorded for future use.
    type_aliases_append(&typechecker->type_aliases, (TypeAlias){.name = node->name, .type = aliased_type});
    return true;
}

bool typechecker_check_variable_reassignment(Typechecker* typechecker, VariableReassignmentNode* node) {
    // The variable must already be declared.
    DeclaredVariable* variable = declared_variables_find_by_name(typechecker->variables, node->variable_name);
    if (!variable) {
        diagnostic_stream_push(
            &typechecker->diagnostics,
            node->position,
            true,
            "unable to assign value to undeclared variable: '%s'",
            node->variable_name
        );
        return false;
    }

    // The type of the variable must match the type of the value.
    ResolvedType* value_type = typechecker_check_value(typechecker, node->value, variable->type);
    if (!value_type) {
        return false;
    }

    if (variable->type->kind != value_type->kind) {
        // The code generator should allow values to be assigned to pointers.
        bool is_value_to_pointer_assignment = variable->type->is_reference && !value_type->is_reference;

        // Non-optionals should be allowed to be assigned to optionals.
        bool is_non_optional_to_optional_assignment = variable->type->is_optional && !value_type->is_optional;
        LOG_DEBUG("typechecker", "is_non_optional_to_optional_assignment: %d", is_non_optional_to_optional_assignment);

        if (!is_value_to_pointer_assignment && !is_non_optional_to_optional_assignment) {
            diagnostic_stream_push(
                &typechecker->diagnostics,
                node->position,
                true,
                "unable to assign value of type '%s' to variable of type '%s'",
                type_to_string((Type*)value_type),
                type_to_string((Type*)variable->type)
            );

            return false;
        }
    }

    return true;
}

// If this returns TYPE_INVALID, the typechecker failed.
ResolvedType* typechecker_check_value(Typechecker* typechecker, Node* node, ResolvedType* expected_type) {
    switch (node->node_type) {
    case NODE_NUMBER_LITERAL:
        return typechecker_check_number_literal(typechecker, (NumberLiteralNode*)node, expected_type);

    case NODE_STRING_LITERAL:
        return typechecker_check_string_literal(typechecker, (StringLiteralNode*)node);

    case NODE_BOOLEAN_LITERAL:
        return typechecker_check_boolean_literal(typechecker, (BooleanLiteralNode*)node);

    case NODE_IDENTIFIER_REFERENCE:
        return typechecker_check_identifier_reference(typechecker, (IdentifierReferenceNode*)node);

    case NODE_BINARY_OPERATION:
        return typechecker_check_binary_operation(typechecker, (BinaryOperationNode*)node, expected_type);

    case NODE_FUNCTION_CALL:
        return typechecker_check_function_call(typechecker, (FunctionCallNode*)node);

    case NODE_FORCE_UNWRAP:
        return typechecker_check_force_unwrap(typechecker, (ForceUnwrapNode*)node, expected_type);

    default: {
        diagnostic_stream_push(
            &typechecker->diagnostics,
            node->position,
            true,
            "unable to type-check node as value: '%s'",
            node_to_string(node)
        );

        return 0;
    }
    }
}

ResolvedType* typechecker_check_number_literal(
    Typechecker* typechecker,
    NumberLiteralNode* node,
    ResolvedType* expected_type
) {
    (void)typechecker;

    ResolvedType* value_type;

    // Ensure that only floating point values are assigned to floating point literals.
    if (floor(node->value) != node->value) {
        // If the expected type is f64, we can coerce it to that.
        if (expected_type->kind == TYPE_KIND_FLOAT_64) {
            value_type = type_create_resolved(false, false, TYPE_KIND_FLOAT_64);
        } else {
            value_type = type_create_resolved(false, false, TYPE_KIND_FLOAT_32);
        }
    } else {
        // Integer literals have a default type of i32.
        value_type = type_create_resolved(false, false, TYPE_KIND_INT_32);

        // If the expected type is a supported integer type, we can coerce it to that.
        if (expected_type->kind == TYPE_KIND_INT_8 || expected_type->kind == TYPE_KIND_INT_32 ||
            expected_type->kind == TYPE_KIND_INT_64) {
            LOG_DEBUG(
                "typechecker",
                "expected type for number literal is '%s', coercing to that type as it is safe to do so",
                type_to_string((Type*)expected_type)
            );

            value_type = type_create_resolved(false, false, expected_type->kind);
        }
    }

    // Assign the new type to the node.
    node->type = (Type*)value_type;
    return value_type;
}

ResolvedType* typechecker_check_string_literal(Typechecker* typechecker, StringLiteralNode* node) {
    (void)typechecker;
    (void)node;

    // String literals are always i8 pointers.
    return type_create_resolved(false, true, TYPE_KIND_INT_8);
}

ResolvedType* typechecker_check_boolean_literal(Typechecker* typechecker, BooleanLiteralNode* node) {
    (void)typechecker;
    (void)node;

    // Boolean literals are always `bool`.
    return type_create_resolved(false, false, TYPE_KIND_BOOL);
}

ResolvedType* typechecker_check_identifier_reference(Typechecker* typechecker, IdentifierReferenceNode* node) {
    // An identifier reference always refers to a previous variable declaration.
    DeclaredVariable* variable = declared_variables_find_by_name(typechecker->variables, node->name);
    if (!variable) {
        diagnostic_stream_push(
            &typechecker->diagnostics,
            node->position,
            true,
            "undeclared variable: '%s'",
            node->name
        );
        return 0;
    }

    // If this identifier reference node is by reference, the type must be a pointer.
    if (node->by_reference) {
        return type_create_resolved(variable->type->is_optional, true, variable->type->kind);
    }

    // The type of this identifier reference is the type of the variable.
    return variable->type;
}

ResolvedType* typechecker_check_binary_operation(
    Typechecker* typechecker,
    BinaryOperationNode* node,
    ResolvedType* expected_type
) {
    // Both sides of the operation must be the same type.
    // This could be expanded to "compatible" types in the future, but for now, we will just make sure
    // that they are the same type.
    ResolvedType* left_type = typechecker_check_value(typechecker, node->left, expected_type);
    if (!left_type) {
        return 0;
    }

    ResolvedType* right_type = typechecker_check_value(typechecker, node->right, expected_type);
    if (!right_type) {
        return 0;
    }

    if (!type_equal((Type*)left_type, (Type*)right_type)) {
        diagnostic_stream_push(
            &typechecker->diagnostics,
            node->position,
            true,
            "incompatible types for binary operation: '%s' and '%s'",
            type_to_string((Type*)left_type),
            type_to_string((Type*)right_type)
        );

        return 0;
    }

    node->type = (Type*)left_type;
    return left_type;
}

ResolvedType* typechecker_check_function_call(Typechecker* typechecker, FunctionCallNode* node) {
    // A function call always refers to a previously declared function.
    DeclaredFunction* function = declared_functions_find_by_name(typechecker->functions, node->name);
    if (!function) {
        diagnostic_stream_push(
            &typechecker->diagnostics,
            node->position,
            true,
            "undeclared function: '%s'",
            node->name
        );
        return 0;
    }

    // FIXME: Allow for function overloading.
    if (node->arguments.length != function->parameters.length) {
        diagnostic_stream_push(
            &typechecker->diagnostics,
            node->position,
            true,
            "function '%s' expects %d parameters, but %d arguments were passed",
            function->name,
            function->parameters.length,
            node->arguments.length
        );

        return 0;
    }

    // Ensure that the arguments passed match the function's parameters.
    for (size_t i = 0; i < function->parameters.length; i++) {
        Parameter parameter = function->parameters.data[i];
        Node* argument = node->arguments.data[i];

        ResolvedType* argument_type = typechecker_check_value(typechecker, argument, (ResolvedType*)parameter.type);
        if (!argument_type) {
            return 0;
        }

        // If the parameter's type does not match the argument passed, this is a problem.
        if (!type_equal((Type*)argument_type, parameter.type)) {
            diagnostic_stream_push(
                &typechecker->diagnostics,
                argument->position,
                true,
                "type '%s' cannot be passed to a function with parameter of type '%s'",
                type_to_string((Type*)argument_type),
                type_to_string(parameter.type)
            );

            return 0;
        }
    }

    // The type of this function call is the return type of the function.
    return function->return_type;
}

ResolvedType* typechecker_check_force_unwrap(
    Typechecker* typechecker,
    ForceUnwrapNode* node,
    ResolvedType* expected_type
) {
    // The value must have a resolvable type.
    ResolvedType* value_type = typechecker_check_value(typechecker, node->value, expected_type);
    if (!value_type) {
        return 0;
    }

    // The value's type must be optional.
    if (!value_type->is_optional) {
        diagnostic_stream_push(
            &typechecker->diagnostics,
            node->value->position,
            true,
            "unable to unwrap non-optional type: '%s'",
            type_to_string((Type*)value_type)
        );

        return 0;
    }

    // If this node is force-unwrapped, the type is the same, except for the fact that it is non-optional.
    return type_create_resolved(false, value_type->is_reference, value_type->kind);
}

void typechecker_destroy(Typechecker* typechecker) {
    diagnostic_stream_destroy(&typechecker->diagnostics);
    declared_variables_destroy(&typechecker->variables);
    declared_functions_destroy(&typechecker->functions);
    type_aliases_destroy(&typechecker->type_aliases);
}
