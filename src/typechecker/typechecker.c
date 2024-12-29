#include "typechecker.h"
#include "../ast/node/function_declaration.h"
#include "../ast/node/identifier_reference.h"
#include "../ast/node/number_literal.h"
#include "../ast/node/return.h"
#include "../ast/node/variable_declaration.h"
#include "declared_variable.h"
#include <math.h>
#include <stdbool.h>

// Forward declarations:
bool typechecker_check(Typechecker* typechecker, NodeStream* node_stream, Type return_type);

bool typechecker_check_statement(Typechecker* typechecker, Node* node, Type return_type);
bool typechecker_check_function_declaration(Typechecker* typechecker, FunctionDeclarationNode* node);
bool typechecker_check_variable_declaration(Typechecker* typechecker, VariableDeclarationNode* node);
bool typechecker_check_return(Typechecker* typechecker, ReturnNode* node, Type return_type);

Type typechecker_check_value(Typechecker* typechecker, Node* value, Type expected_type);
Type typechecker_check_number_literal(Typechecker* typechecker, NumberLiteralNode* node, Type expected_type);
Type typechecker_check_identifier_reference(Typechecker* typechecker, IdentifierReferenceNode* node);

Typechecker typechecker_create() {
    DiagnosticStream diagnostics;
    diagnostic_stream_initialize(&diagnostics, 1);

    DeclaredVariables variables;
    declared_variables_initialize(&variables, 1);

    return (Typechecker){diagnostics, variables};
}

void typechecker_run(Typechecker* typechecker, NodeStream* node_stream) {
    typechecker_check(typechecker, node_stream, TYPE_INVALID);
}

bool typechecker_check(Typechecker* typechecker, NodeStream* node_stream, Type return_type) {
    for (size_t i = 0; i < node_stream->length; i++) {
        Node* node = node_stream->data[i];
        if (!typechecker_check_statement(typechecker, node, return_type)) {
            return false;
        }
    }

    return true;
}

bool typechecker_check_statement(Typechecker* typechecker, Node* node, Type return_type) {
    switch (node->node_type) {
    case NODE_FUNCTION_DECLARATION:
        return typechecker_check_function_declaration(typechecker, (FunctionDeclarationNode*)node);

    case NODE_VARIABLE_DECLARATION:
        return typechecker_check_variable_declaration(typechecker, (VariableDeclarationNode*)node);

    case NODE_RETURN:
        return typechecker_check_return(typechecker, (ReturnNode*)node, return_type);

    default: {
        diagnostic_stream_push(&typechecker->diagnostics, node->position, true, "unable to type-check node: '%s'",
                               node_to_string(node));

        return false;
    }
    }
}

bool typechecker_check_function_declaration(Typechecker* typechecker, FunctionDeclarationNode* node) {
    if (node->is_external) {
        // External functions have nothing to typecheck at the moment.
        LOG_DEBUG("typechecker", "skipping external function: '%s'", node->name);
        return true;
    }

    // If this is not an external function, and there is no function body, that should not be supported.
    if (!node->function_body) {
        diagnostic_stream_push(&typechecker->diagnostics, node->position, true,
                               "non-external function '%s' has no function body", node->name);
        return false;
    }

    LOG_DEBUG("typechecker", "typechecking function: '%s'", node->name);

    // Before doing any typechecking, we should treat this as a new scope.
    declared_variables_destroy(&typechecker->variables);
    declared_variables_initialize(&typechecker->variables, 1);

    // Function declarations don't have much to typecheck, their parameters should already have types defined,
    // so all we have to do is verify that the function body is OK.
    return typechecker_check(typechecker, &node->function_body->body, node->return_type);
}

bool typechecker_check_variable_declaration(Typechecker* typechecker, VariableDeclarationNode* node) {
    // A variable declaration always has an expected type.
    // If the value does not match the expected type, we must throw an error.
    Type value_type = typechecker_check_value(typechecker, node->value, node->type);
    if (value_type.kind == TYPE_KIND_INVALID) {
        return false;
    }

    if (value_type.kind != node->type.kind || value_type.is_pointer != node->type.is_pointer) {
        // These types are not matching!
        diagnostic_stream_push(&typechecker->diagnostics, node->position, true,
                               "unable to assign value of type '%s' to type '%s'", type_to_string(value_type),
                               type_to_string(node->type));
        return false;
    }

    // Record this as a declared variable within this scope.
    declared_variables_append(&typechecker->variables, (DeclaredVariable){.name = node->name, .type = value_type});
    return true;
}

bool typechecker_check_return(Typechecker* typechecker, ReturnNode* node, Type return_type) {
    if (return_type.kind == TYPE_KIND_INVALID) {
        diagnostic_stream_push(&typechecker->diagnostics, node->position, true,
                               "unable to type check return statement, current function has no known return type?");
        return false;
    }

    // The value within a return node must match the return type of the function.
    Type value_type = typechecker_check_value(typechecker, node->value, return_type);
    if (value_type.kind != return_type.kind || value_type.is_pointer != return_type.is_pointer) {
        // These types are not matching!
        diagnostic_stream_push(&typechecker->diagnostics, node->position, true,
                               "type '%s' cannot be returned from a function with return type '%s'",
                               type_to_string(value_type), type_to_string(return_type));
        return false;
    }

    return true;
}

// If this returns TYPE_INVALID, the typechecker failed.
Type typechecker_check_value(Typechecker* typechecker, Node* node, Type expected_type) {
    switch (node->node_type) {
    case NODE_NUMBER_LITERAL:
        return typechecker_check_number_literal(typechecker, (NumberLiteralNode*)node, expected_type);

    case NODE_IDENTIFIER_REFERENCE:
        return typechecker_check_identifier_reference(typechecker, (IdentifierReferenceNode*)node);

    default: {
        diagnostic_stream_push(&typechecker->diagnostics, node->position, true,
                               "unable to type-check node as value: '%s'", node_to_string(node));

        return TYPE_INVALID;
    }
    }
}

Type typechecker_check_number_literal(Typechecker* typechecker, NumberLiteralNode* node, Type expected_type) {
    (void)typechecker;

    Type value_type;

    // If this is a floating point number, f32 must be used.
    if (floor(node->value) != node->value) {
        value_type = type_create(TYPE_KIND_FLOAT_32, false);
    } else {
        // Integer literals have a default type of i32.
        value_type = type_create(TYPE_KIND_INT_32, false);

        // If the expected type is a supported integer type, we can coerce it to that.
        if (expected_type.kind == TYPE_KIND_INT_8 || expected_type.kind == TYPE_KIND_INT_32 ||
            expected_type.kind == TYPE_KIND_INT_64) {
            LOG_DEBUG("typechecker",
                      "expected type for number literal is '%s', coercing to that type as it is safe to do so",
                      type_to_string(expected_type));
            value_type = expected_type;
        }
    }

    // Assign the new type to the node.
    node->expected_type = value_type;
    return value_type;
}

Type typechecker_check_identifier_reference(Typechecker* typechecker, IdentifierReferenceNode* node) {
    // An identifier reference always refers to a previous variable declaration.
    DeclaredVariable* variable = declared_variables_find_by_name(typechecker->variables, node->name);
    if (!variable) {
        diagnostic_stream_push(&typechecker->diagnostics, node->position, true, "undeclared variable: '%s'",
                               node->name);
        return TYPE_INVALID;
    }

    // The type of this identifier reference is the type of the variable.
    return variable->type;
}

void typechecker_destroy(Typechecker* typechecker) {
    diagnostic_stream_destroy(&typechecker->diagnostics);
    declared_variables_destroy(&typechecker->variables);
}
